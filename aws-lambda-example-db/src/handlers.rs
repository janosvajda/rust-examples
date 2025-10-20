use std::sync::Arc;

use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{
    http::{Method, StatusCode},
    Body, Error as LambdaError, Request, RequestExt, RequestPayloadExt, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, warn};

use crate::{
    auth::{
        current_epoch_seconds, generate_refresh_token, hash_password, issue_jwt, verify_password,
        ACCESS_TOKEN_TTL_SECONDS, REFRESH_TOKEN_TTL_SECONDS,
    },
    context::AppContext,
    error::{lambda_error, AppError},
    user::{CreateUserPayload, UserRecord},
};

/// Top-level request dispatcher used by the Lambda runtime.
pub async fn handle_request(
    ctx: Arc<AppContext>,
    event: Request,
) -> Result<Response<Body>, LambdaError> {
    let path = event.uri().path();
    match (event.method().clone(), path) {
        (Method::POST, "/users") => create_user(ctx.as_ref(), event).await,
        (Method::GET, "/users") => get_user(ctx.as_ref(), event).await,
        (Method::POST, "/login") => login_user(ctx.as_ref(), event).await,
        (Method::POST, "/token/refresh") => refresh_access_token(ctx.as_ref(), event).await,
        (Method::POST, "/token/revoke") => revoke_refresh_token(ctx.as_ref(), event).await,
        _ => Ok(json_response(
            StatusCode::NOT_FOUND,
            json!({ "message": "Unsupported route" }),
        )),
    }
}

async fn create_user(ctx: &AppContext, event: Request) -> Result<Response<Body>, LambdaError> {
    let payload = match event.payload::<CreateUserPayload>().unwrap_or_else(|e| {
        warn!("failed to parse payload: {e:?}");
        None
    }) {
        Some(p) => p,
        None => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "invalid JSON payload" }),
            ))
        }
    };
    let is_update = payload.user_id.is_some();
    let family_id = payload.family_id.clone();
    let user_name = payload.user_name.clone();
    let email = payload.email.clone();

    if !is_update {
        // Ensure email is unique.
        let existing_credentials = ctx
            .client()
            .get_item()
            .table_name(ctx.credentials_table())
            .key("email", AttributeValue::S(email.clone()))
            .send()
            .await
            .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;
        if existing_credentials.item.is_some() {
            return Ok(json_response(
                StatusCode::CONFLICT,
                json!({ "message": format!("email `{email}` is already registered") }),
            ));
        }
    }

    if !is_update {
        let duplicate = ctx
            .client()
            .query()
            .table_name(ctx.table_name())
            .index_name("FamilyUserIndex")
            .key_condition_expression("#fid = :fid AND #uname = :uname")
            .expression_attribute_names("#fid", "familyId")
            .expression_attribute_names("#uname", "userName")
            .expression_attribute_values(":fid", AttributeValue::S(family_id.clone()))
            .expression_attribute_values(":uname", AttributeValue::S(user_name.clone()))
            .limit(1)
            .send()
            .await
            .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;
        if duplicate.count > 0 {
            return Ok(json_response(
                StatusCode::CONFLICT,
                json!({ "message": format!("user `{}` already exists for family `{}`", user_name, family_id) }),
            ));
        }
    }

    let password_hash = hash_password(&payload.password).map_err(lambda_error)?;

    let record = UserRecord::new(payload);

    if !is_update {
        ctx.client()
            .put_item()
            .table_name(ctx.credentials_table())
            .item("email", AttributeValue::S(email.clone()))
            .item("userId", AttributeValue::S(record.user_id.clone()))
            .item("familyId", AttributeValue::S(record.family_id.clone()))
            .item("passwordHash", AttributeValue::S(password_hash.clone()))
            .condition_expression("attribute_not_exists(email)")
            .send()
            .await
            .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;
    } else {
        ctx.client()
            .update_item()
            .table_name(ctx.credentials_table())
            .key("email", AttributeValue::S(email.clone()))
            .update_expression("SET passwordHash = :hash, familyId = :fid, userId = :uid")
            .expression_attribute_values(":hash", AttributeValue::S(password_hash.clone()))
            .expression_attribute_values(":fid", AttributeValue::S(record.family_id.clone()))
            .expression_attribute_values(":uid", AttributeValue::S(record.user_id.clone()))
            .condition_expression("attribute_exists(email)")
            .send()
            .await
            .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;
    }

    ctx.client()
        .put_item()
        .table_name(ctx.table_name())
        .set_item(Some(record.clone().into_item()))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    Ok(json_response(
        StatusCode::CREATED,
        serde_json::to_value(&record).unwrap_or_else(|_| json!({})),
    ))
}

async fn get_user(ctx: &AppContext, event: Request) -> Result<Response<Body>, LambdaError> {
    let user_id = match event
        .query_string_parameters_ref()
        .and_then(|qs| qs.first("userId"))
    {
        Some(id) => id.to_owned(),
        None => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "userId query parameter is required" }),
            ))
        }
    };

    let output = ctx
        .client()
        .get_item()
        .table_name(ctx.table_name())
        .key("userId", AttributeValue::S(user_id.clone()))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    if let Some(item) = output.item {
        let record = UserRecord::from_item(item).map_err(lambda_error)?;
        Ok(json_response(
            StatusCode::OK,
            serde_json::to_value(record).unwrap_or_else(|_| json!({})),
        ))
    } else {
        Ok(json_response(
            StatusCode::NOT_FOUND,
            json!({ "message": format!("user `{user_id}` not found") }),
        ))
    }
}

#[derive(Deserialize)]
struct LoginPayload {
    email: String,
    password: String,
}

async fn login_user(ctx: &AppContext, event: Request) -> Result<Response<Body>, LambdaError> {
    let payload = match event.payload::<LoginPayload>().unwrap_or_else(|e| {
        warn!("failed to parse login payload: {e:?}");
        None
    }) {
        Some(p) => p,
        None => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "invalid JSON payload" }),
            ))
        }
    };

    let credentials = ctx
        .client()
        .get_item()
        .table_name(ctx.credentials_table())
        .key("email", AttributeValue::S(payload.email.clone()))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    let item = match credentials.item {
        Some(item) => item,
        None => {
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                json!({ "message": "invalid credentials" }),
            ))
        }
    };

    let stored_hash = item
        .get("passwordHash")
        .and_then(|attr| attr.as_s().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("credential missing passwordHash".into())))?;

    if !verify_password(&payload.password, stored_hash).map_err(lambda_error)? {
        return Ok(json_response(
            StatusCode::UNAUTHORIZED,
            json!({ "message": "invalid credentials" }),
        ));
    }

    let user_id = item
        .get("userId")
        .and_then(|attr| attr.as_s().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("credential missing userId".into())))?;
    let family_id = item
        .get("familyId")
        .and_then(|attr| attr.as_s().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("credential missing familyId".into())))?;

    let token = issue_jwt(
        ctx.jwt_secret(),
        user_id,
        family_id,
        ACCESS_TOKEN_TTL_SECONDS,
    )
    .map_err(lambda_error)?;

    // Ensure only one active refresh token per user by removing existing entries first.
    let existing_tokens = ctx
        .client()
        .query()
        .table_name(ctx.refresh_table())
        .index_name("FamilyUserIndex")
        .key_condition_expression("#fid = :fid AND #uid = :uid")
        .expression_attribute_names("#fid", "familyId")
        .expression_attribute_names("#uid", "userId")
        .expression_attribute_values(":fid", AttributeValue::S(family_id.to_string()))
        .expression_attribute_values(":uid", AttributeValue::S(user_id.to_string()))
        .projection_expression("refreshToken")
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    let tokens_to_delete: Vec<String> = existing_tokens
        .items()
        .iter()
        .filter_map(|token_item| {
            token_item
                .get("refreshToken")
                .and_then(|attr| attr.as_s().ok())
                .map(|s| s.to_string())
        })
        .collect();

    for refresh in tokens_to_delete {
        let _ = ctx
            .client()
            .delete_item()
            .table_name(ctx.refresh_table())
            .key("refreshToken", AttributeValue::S(refresh))
            .send()
            .await;
    }

    let refresh_token = generate_refresh_token();
    let now = current_epoch_seconds().map_err(lambda_error)?;
    let refresh_exp = now + REFRESH_TOKEN_TTL_SECONDS as i64;
    ctx.client()
        .put_item()
        .table_name(ctx.refresh_table())
        .item("refreshToken", AttributeValue::S(refresh_token.clone()))
        .item("userId", AttributeValue::S(user_id.to_string()))
        .item("familyId", AttributeValue::S(family_id.to_string()))
        .item("expiresAt", AttributeValue::N(refresh_exp.to_string()))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "accessToken": token,
            "tokenType": "Bearer",
            "expiresIn": ACCESS_TOKEN_TTL_SECONDS,
            "userId": user_id,
            "familyId": family_id,
            "refreshToken": refresh_token,
            "refreshExpiresIn": REFRESH_TOKEN_TTL_SECONDS,
        }),
    ))
}

#[derive(Deserialize)]
struct RefreshPayload {
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

async fn refresh_access_token(
    ctx: &AppContext,
    event: Request,
) -> Result<Response<Body>, LambdaError> {
    let payload = match event.payload::<RefreshPayload>().unwrap_or_else(|e| {
        warn!("failed to parse refresh payload: {e:?}");
        None
    }) {
        Some(p) => p,
        None => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "invalid JSON payload" }),
            ))
        }
    };

    let item = ctx
        .client()
        .get_item()
        .table_name(ctx.refresh_table())
        .key(
            "refreshToken",
            AttributeValue::S(payload.refresh_token.clone()),
        )
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?
        .item;

    let item = match item {
        Some(item) => item,
        None => {
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                json!({ "message": "invalid refresh token" }),
            ))
        }
    };

    let expires_at = item
        .get("expiresAt")
        .and_then(|attr| attr.as_n().ok())
        .and_then(|n| n.parse::<i64>().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("refresh token missing expiresAt".into())))?;
    let now = current_epoch_seconds().map_err(lambda_error)?;
    if now >= expires_at {
        let _ = ctx
            .client()
            .delete_item()
            .table_name(ctx.refresh_table())
            .key("refreshToken", AttributeValue::S(payload.refresh_token))
            .send()
            .await;
        return Ok(json_response(
            StatusCode::UNAUTHORIZED,
            json!({ "message": "refresh token expired" }),
        ));
    }

    let user_id = item
        .get("userId")
        .and_then(|attr| attr.as_s().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("refresh token missing userId".into())))?;
    let family_id = item
        .get("familyId")
        .and_then(|attr| attr.as_s().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("refresh token missing familyId".into())))?;

    ctx.client()
        .delete_item()
        .table_name(ctx.refresh_table())
        .key("refreshToken", AttributeValue::S(payload.refresh_token))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    let access_token = issue_jwt(
        ctx.jwt_secret(),
        user_id,
        family_id,
        ACCESS_TOKEN_TTL_SECONDS,
    )
    .map_err(lambda_error)?;

    let new_refresh_token = generate_refresh_token();
    let new_exp = now + REFRESH_TOKEN_TTL_SECONDS as i64;
    ctx.client()
        .put_item()
        .table_name(ctx.refresh_table())
        .item("refreshToken", AttributeValue::S(new_refresh_token.clone()))
        .item("userId", AttributeValue::S(user_id.to_string()))
        .item("familyId", AttributeValue::S(family_id.to_string()))
        .item("expiresAt", AttributeValue::N(new_exp.to_string()))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "accessToken": access_token,
            "tokenType": "Bearer",
            "expiresIn": ACCESS_TOKEN_TTL_SECONDS,
            "refreshToken": new_refresh_token,
            "refreshExpiresIn": REFRESH_TOKEN_TTL_SECONDS,
            "userId": user_id,
            "familyId": family_id,
        }),
    ))
}

async fn revoke_refresh_token(
    ctx: &AppContext,
    event: Request,
) -> Result<Response<Body>, LambdaError> {
    let payload = match event.payload::<RefreshPayload>().unwrap_or_else(|e| {
        warn!("failed to parse revoke payload: {e:?}");
        None
    }) {
        Some(p) => p,
        None => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "invalid JSON payload" }),
            ))
        }
    };

    ctx.client()
        .delete_item()
        .table_name(ctx.refresh_table())
        .key("refreshToken", AttributeValue::S(payload.refresh_token))
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    Ok(json_response(StatusCode::OK, json!({ "revoked": true })))
}
fn json_response<T: Serialize>(status: StatusCode, value: T) -> Response<Body> {
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());

    if status.is_server_error() {
        error!(
            http_status = status.as_u16(),
            body = %body,
            "returning server error response"
        );
    } else if status.is_client_error() {
        warn!(
            http_status = status.as_u16(),
            body = %body,
            "returning client error response"
        );
    }

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(body))
        .expect("failed to build response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn json_response_sets_content_type() {
        let response = json_response(StatusCode::OK, json!({ "ok": true }));
        assert_eq!(response.status(), StatusCode::OK);
        let header = response.headers().get("content-type").unwrap();
        assert_eq!(header, "application/json");
    }
}
