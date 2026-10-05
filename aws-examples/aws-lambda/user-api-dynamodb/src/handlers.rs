//! HTTP handlers backing the Lambda entrypoint.
//!
//! The Lambda is exposed through API Gateway and speaks a simple JSON-over-HTTP
//! protocol. Each handler performs three broad steps:
//!   1. Deserialise the request payload or query parameters.
//!   2. Interact with DynamoDB / SSM via the shared `AppContext`.
//!   3. Return an HTTP response (or propagate an error which the runtime converts
//!      to a 500).
//!
//! Because API Gateway prepends the stage (e.g., `/Prod`) to the incoming path we
//! rely on `AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH=true` (set in the template) so
//! the `lambda_http` crate strips that prefix before dispatching below.

use std::sync::Arc;

use aws_sdk_dynamodb::types::{AttributeValue, Delete, Put, TransactWriteItem, Update};
use lambda_http::{
    Body, Error as LambdaError, Request, RequestExt, RequestPayloadExt, Response,
    http::{Method, StatusCode},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, warn};

use crate::{
    auth::{
        ACCESS_TOKEN_TTL_SECONDS, Identity, REFRESH_TOKEN_TTL_SECONDS, current_epoch_seconds,
        generate_refresh_token, hash_password, issue_jwt, verify_jwt, verify_password,
        verify_password_for_unknown_user,
    },
    context::AppContext,
    error::{AppError, lambda_error},
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

/// Checks the `Authorization: Bearer <token>` header. Returns the identity the
/// token proves, or the 401 response to send back.
fn authenticate(ctx: &AppContext, event: &Request) -> Result<Identity, Box<Response<Body>>> {
    let token = event
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let unauthorized = || {
        Box::new(json_response(
            StatusCode::UNAUTHORIZED,
            json!({ "message": "a valid access token is required" }),
        ))
    };
    let token = token.ok_or_else(unauthorized)?;
    verify_jwt(ctx.jwt_secret(), token).map_err(|_| unauthorized())
}

fn forbidden() -> Response<Body> {
    json_response(
        StatusCode::FORBIDDEN,
        json!({ "message": "this token may only access its own user" }),
    )
}

/// Checks the registration fields. Returns a message for the first problem found.
fn validate(payload: &CreateUserPayload) -> Result<(), &'static str> {
    let email_ok = payload.email.len() <= 254
        && !payload.email.chars().any(char::is_whitespace)
        && payload.email.split_once('@').is_some_and(|(name, domain)| {
            !name.is_empty()
                && domain.contains('.')
                && !domain.contains('@')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
        });
    if !email_ok {
        return Err("email must look like name@example.com");
    }
    // A minimum against guessing; a maximum so nobody can make Argon2 hash megabytes.
    if !(8..=1024).contains(&payload.password.len()) {
        return Err("password must be 8 to 1024 bytes long");
    }
    if !(1..=100).contains(&payload.user_name.len())
        || payload.user_name.trim().is_empty()
        || !(1..=100).contains(&payload.family_id.len())
        || payload.family_id.trim().is_empty()
    {
        return Err(
            "userName and familyId must be 1 to 100 UTF-8 bytes and contain non-whitespace",
        );
    }
    Ok(())
}

/// Run password work away from async scheduler threads with bounded concurrency.
async fn password_work<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> Result<T, LambdaError> {
    static LIMIT: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
    let permit = LIMIT.acquire().await?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        work()
    })
    .await?
    .map_err(lambda_error)
}

/// Length-prefixed keys distinguish (family, name) pairs unambiguously.
/// Reservation rows have no GSI keys and do not appear in user listings.
fn name_key(family: &str, name: &str) -> String {
    format!("NAME#{}:{family}{name}", family.len())
}

/// Commit the user, credentials and unique-name reservation together.
async fn create_user(ctx: &AppContext, event: Request) -> Result<Response<Body>, LambdaError> {
    let Some(payload) = event.payload::<CreateUserPayload>().unwrap_or(None) else {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            json!({ "message": "invalid JSON payload" }),
        ));
    };
    if let Err(message) = validate(&payload) {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            json!({ "message": message }),
        ));
    }
    if let Some(user_id) = &payload.user_id {
        let identity = match authenticate(ctx, &event) {
            Ok(identity) => identity,
            Err(response) => return Ok(*response),
        };
        if &identity.user_id != user_id {
            return Ok(forbidden());
        }
    }
    let is_update = payload.user_id.is_some();
    let password = payload.password.clone();
    let mut record = UserRecord::new(payload);
    let mut previous = None;
    let mut revision = 0_u64;
    if is_update {
        let existing = ctx
            .client()
            .get_item()
            .table_name(ctx.table_name())
            .key("userId", AttributeValue::S(record.user_id.clone()))
            .consistent_read(true)
            .send()
            .await?
            .item;
        let Some(existing) = existing else {
            return Ok(json_response(
                StatusCode::NOT_FOUND,
                json!({ "message": "user not found" }),
            ));
        };
        revision = existing
            .get("revision")
            .map(|value| {
                value
                    .as_n()
                    .ok()
                    .and_then(|n| n.parse::<u64>().ok())
                    .ok_or_else(|| lambda_error(AppError::Dynamo("invalid user revision".into())))
            })
            .transpose()?
            .unwrap_or(0);
        let old = UserRecord::from_item(existing).map_err(lambda_error)?;
        if old.email != record.email {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "updates cannot change email" }),
            ));
        }
        record.created_at = old.created_at;
        previous = Some(old);
    }
    let password_hash = password_work(move || hash_password(&password)).await?;
    let next_revision = revision.checked_add(1).ok_or("user revision exhausted")?;
    let mut item = record.clone().into_item();
    item.insert(
        "revision".into(),
        AttributeValue::N(next_revision.to_string()),
    );
    let mut user_write = Put::builder()
        .table_name(ctx.table_name())
        .set_item(Some(item));
    if is_update {
        user_write = user_write.condition_expression(if revision == 0 {
            "attribute_exists(userId) AND attribute_not_exists(revision)"
        } else {
            "revision = :revision"
        });
        if revision != 0 {
            user_write = user_write
                .expression_attribute_values(":revision", AttributeValue::N(revision.to_string()));
        }
    } else {
        user_write = user_write.condition_expression("attribute_not_exists(userId)");
    }
    let reservation = Put::builder()
        .table_name(ctx.table_name())
        .item(
            "userId",
            AttributeValue::S(name_key(&record.family_id, &record.user_name)),
        )
        .item("ownerId", AttributeValue::S(record.user_id.clone()))
        .condition_expression("attribute_not_exists(userId) OR ownerId = :uid")
        .expression_attribute_values(":uid", AttributeValue::S(record.user_id.clone()))
        .build()?;
    let mut writes = vec![
        TransactWriteItem::builder()
            .put(user_write.build()?)
            .build(),
        TransactWriteItem::builder().put(reservation).build(),
    ];
    if let Some(old) = previous {
        let old_key = name_key(&old.family_id, &old.user_name);
        if old_key != name_key(&record.family_id, &record.user_name) {
            let deletion = Delete::builder()
                .table_name(ctx.table_name())
                .key("userId", AttributeValue::S(old_key))
                .condition_expression("attribute_not_exists(userId) OR ownerId = :uid")
                .expression_attribute_values(":uid", AttributeValue::S(record.user_id.clone()))
                .build()?;
            writes.push(TransactWriteItem::builder().delete(deletion).build());
        }
        let credentials = Update::builder()
            .table_name(ctx.credentials_table())
            .key("email", AttributeValue::S(record.email.clone()))
            .update_expression("SET passwordHash = :hash, familyId = :fid")
            .condition_expression("userId = :uid")
            .expression_attribute_values(":hash", AttributeValue::S(password_hash))
            .expression_attribute_values(":fid", AttributeValue::S(record.family_id.clone()))
            .expression_attribute_values(":uid", AttributeValue::S(record.user_id.clone()))
            .build()?;
        writes.push(TransactWriteItem::builder().update(credentials).build());
    } else {
        let credentials = Put::builder()
            .table_name(ctx.credentials_table())
            .item("email", AttributeValue::S(record.email.clone()))
            .item("userId", AttributeValue::S(record.user_id.clone()))
            .item("familyId", AttributeValue::S(record.family_id.clone()))
            .item("passwordHash", AttributeValue::S(password_hash))
            .condition_expression("attribute_not_exists(email)")
            .build()?;
        writes.push(TransactWriteItem::builder().put(credentials).build());
    }
    match ctx
        .client()
        .transact_write_items()
        .set_transact_items(Some(writes))
        .send()
        .await
    {
        Ok(_) => Ok(json_response(StatusCode::CREATED, &record)),
        Err(e)
            if matches!(e.as_service_error(),
            Some(aws_sdk_dynamodb::operation::transact_write_items::TransactWriteItemsError::TransactionCanceledException(reason))
            if reason.cancellation_reasons().iter().any(|r| matches!(r.code(), Some("ConditionalCheckFailed" | "TransactionConflict")))) =>
        {
            Ok(json_response(
                StatusCode::CONFLICT,
                json!({ "message": "email or family/userName is taken, or the record changed; retry after reading it" }),
            ))
        }
        Err(e) => Err(e.into()),
    }
}

/// Look up a user by `userId`.
///
/// The `userId` is required as a query parameter. The handler performs a
/// straight `GetItem` against the users table and returns a 404-style payload if
/// nothing matches.
async fn get_user(ctx: &AppContext, event: Request) -> Result<Response<Body>, LambdaError> {
    let identity = match authenticate(ctx, &event) {
        Ok(identity) => identity,
        Err(response) => return Ok(*response),
    };
    let user_id = match event
        .query_string_parameters_ref()
        .and_then(|qs| qs.first("userId"))
    {
        Some(id) => id.to_owned(),
        None => {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "userId query parameter is required" }),
            ));
        }
    };
    if identity.user_id != user_id {
        return Ok(forbidden()); // a token only reads its own record
    }

    let output = ctx
        .client()
        .get_item()
        .table_name(ctx.table_name())
        .key("userId", AttributeValue::S(user_id.clone()))
        .consistent_read(true)
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    if let Some(item) = output.item {
        let record = UserRecord::from_item(item).map_err(lambda_error)?;
        Ok(json_response(StatusCode::OK, record))
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

/// Validate credentials and issue a fresh access/refresh token pair.
///
/// After verifying the Argon2 hash, we replace the user's single refresh row and respond with a signed JWT plus the new
/// refresh token metadata.
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
            ));
        }
    };

    if payload.email.is_empty()
        || payload.email.len() > 254
        || !(8..=1024).contains(&payload.password.len())
    {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            json!({ "message": "email/password length is invalid" }),
        ));
    }
    let credentials = ctx
        .client()
        .get_item()
        .table_name(ctx.credentials_table())
        .key("email", AttributeValue::S(payload.email.clone()))
        .consistent_read(true)
        .send()
        .await
        .map_err(|e| lambda_error(AppError::Dynamo(e.to_string())))?;

    let item = match credentials.item {
        Some(item) => item,
        None => {
            let password = payload.password;
            password_work(move || verify_password_for_unknown_user(&password)).await?;
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                json!({ "message": "invalid credentials" }),
            ));
        }
    };

    let stored_hash = item
        .get("passwordHash")
        .and_then(|attr| attr.as_s().ok())
        .ok_or_else(|| lambda_error(AppError::Auth("credential missing passwordHash".into())))?;

    let password = payload.password;
    let stored_hash = stored_hash.clone();
    if !password_work(move || verify_password(&password, &stored_hash)).await? {
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
    let refresh_token = replace_refresh_token(ctx, user_id, family_id).await?;

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

/// The public prefix selects a row; the random suffix remains the credential.
fn refresh_key(token: &str) -> Option<String> {
    if token.len() != 73 {
        return None;
    }
    let (user, secret) = token.split_once('.')?;
    uuid::Uuid::parse_str(user).ok()?;
    uuid::Uuid::parse_str(secret).ok()?;
    Some(format!("ACTIVE#{user}"))
}

/// Rotate one user's single stored token with an atomic conditional replacement.
async fn refresh_access_token(
    ctx: &AppContext,
    event: Request,
) -> Result<Response<Body>, LambdaError> {
    let Some(payload) = event.payload::<RefreshPayload>().unwrap_or(None) else {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            json!({ "message": "invalid JSON payload" }),
        ));
    };
    let invalid = || {
        json_response(
            StatusCode::UNAUTHORIZED,
            json!({ "message": "invalid or expired refresh token" }),
        )
    };
    let Some(key) = refresh_key(&payload.refresh_token) else {
        return Ok(invalid());
    };
    let item = ctx
        .client()
        .get_item()
        .table_name(ctx.refresh_table())
        .key("refreshToken", AttributeValue::S(key))
        .consistent_read(true)
        .send()
        .await?
        .item;
    let Some(item) = item else {
        return Ok(invalid());
    };
    let string = |name: &str| {
        item.get(name)
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| lambda_error(AppError::Auth(format!("refresh row missing {name}"))))
    };
    let expires_at = item
        .get("expiresAt")
        .and_then(|v| v.as_n().ok())
        .and_then(|n| n.parse::<i64>().ok())
        .ok_or("invalid refresh expiry")?;
    if string("token")? != &payload.refresh_token
        || current_epoch_seconds().map_err(lambda_error)? >= expires_at
    {
        return Ok(invalid());
    }
    let user_id = string("userId")?;
    let family_id = string("familyId")?;
    let access_token = issue_jwt(
        ctx.jwt_secret(),
        user_id,
        family_id,
        ACCESS_TOKEN_TTL_SECONDS,
    )
    .map_err(lambda_error)?;
    let Some(next) =
        write_refresh_token(ctx, user_id, family_id, Some(&payload.refresh_token)).await?
    else {
        return Ok(invalid());
    };
    Ok(json_response(
        StatusCode::OK,
        json!({ "accessToken": access_token, "tokenType": "Bearer", "expiresIn": ACCESS_TOKEN_TTL_SECONDS,
        "refreshToken": next, "refreshExpiresIn": REFRESH_TOKEN_TTL_SECONDS, "userId": user_id, "familyId": family_id }),
    ))
}

/// Idempotent logout: an old token cannot delete a newer login's token.
async fn revoke_refresh_token(
    ctx: &AppContext,
    event: Request,
) -> Result<Response<Body>, LambdaError> {
    let Some(payload) = event.payload::<RefreshPayload>().unwrap_or(None) else {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            json!({ "message": "invalid JSON payload" }),
        ));
    };
    if let Some(key) = refresh_key(&payload.refresh_token) {
        let result = ctx
            .client()
            .delete_item()
            .table_name(ctx.refresh_table())
            .key("refreshToken", AttributeValue::S(key))
            .condition_expression("#token = :token")
            .expression_attribute_names("#token", "token")
            .expression_attribute_values(":token", AttributeValue::S(payload.refresh_token))
            .send()
            .await;
        if let Err(e) = result
            && !e
                .as_service_error()
                .is_some_and(|s| s.is_conditional_check_failed_exception())
        {
            return Err(e.into());
        }
    }
    Ok(json_response(StatusCode::OK, json!({ "revoked": true })))
}

async fn replace_refresh_token(
    ctx: &AppContext,
    user: &str,
    family: &str,
) -> Result<String, LambdaError> {
    write_refresh_token(ctx, user, family, None)
        .await?
        .ok_or_else(|| "unconditional token write failed".into())
}

/// A stable primary key means one row per user, including concurrent logins.
/// Rotation checks the old credential and expiry in the SAME write.
async fn write_refresh_token(
    ctx: &AppContext,
    user: &str,
    family: &str,
    old: Option<&str>,
) -> Result<Option<String>, LambdaError> {
    let token = format!("{user}.{}", generate_refresh_token());
    let now = current_epoch_seconds().map_err(lambda_error)?;
    let expires = now
        .checked_add(REFRESH_TOKEN_TTL_SECONDS as i64)
        .ok_or("refresh expiry overflow")?;
    let mut request = ctx
        .client()
        .put_item()
        .table_name(ctx.refresh_table())
        .item("refreshToken", AttributeValue::S(format!("ACTIVE#{user}")))
        .item("token", AttributeValue::S(token.clone()))
        .item("userId", AttributeValue::S(user.to_owned()))
        .item("familyId", AttributeValue::S(family.to_owned()))
        .item("expiresAt", AttributeValue::N(expires.to_string()));
    if let Some(old) = old {
        request = request
            .condition_expression("#token = :old AND expiresAt > :now")
            .expression_attribute_names("#token", "token")
            .expression_attribute_values(":old", AttributeValue::S(old.to_owned()))
            .expression_attribute_values(":now", AttributeValue::N(now.to_string()));
    }
    match request.send().await {
        Ok(_) => Ok(Some(token)),
        Err(e)
            if e.as_service_error()
                .is_some_and(|s| s.is_conditional_check_failed_exception()) =>
        {
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}
fn json_response<T: Serialize>(status: StatusCode, value: T) -> Response<Body> {
    let (status, body) = match serde_json::to_string(&value) {
        Ok(body) => (status, body),
        Err(error) => {
            error!(%error, "failed to serialize response");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                r#"{"message":"response serialization failed"}"#.to_owned(),
            )
        }
    };

    // Log the status, never the body: bodies contain email addresses and other
    // personal data, which don't belong in logs.
    if status.is_server_error() {
        error!(
            http_status = status.as_u16(),
            "returning server error response"
        );
    } else if status.is_client_error() {
        warn!(
            http_status = status.as_u16(),
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

    #[test]
    fn serialization_failure_returns_server_error() {
        struct BrokenResponse;
        impl Serialize for BrokenResponse {
            fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("unavailable response"))
            }
        }
        let response = json_response(StatusCode::OK, BrokenResponse);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let Body::Text(body) = response.body() else {
            panic!("expected JSON text")
        };
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(body).unwrap(),
            json!({ "message": "response serialization failed" })
        );
    }

    #[tokio::test]
    async fn json_response_sets_content_type() {
        let response = json_response(StatusCode::OK, json!({ "ok": true }));
        assert_eq!(response.status(), StatusCode::OK);
        let header = response.headers().get("content-type").unwrap();
        assert_eq!(header, "application/json");
    }
}
