mod common;

use anyhow::Result;
use lambda_http::{self, Body};
use serde_json::json;
use uuid::Uuid;

use common::{body_as_string, setup_environment};

#[tokio::test]
async fn refresh_and_revoke_flow() -> Result<()> {
    let Some(setup) = setup_environment().await else {
        return Ok(());
    };
    let ctx = setup.ctx.clone();

    let family_id = format!("family-{}", Uuid::new_v4().simple());
    let create_payload = json!({
        "userName": "integration-user",
        "email": "integration@example.com",
        "password": "secret",
        "familyId": family_id
    });
    let create_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::Text(create_payload.to_string()))
        .expect("create request");
    aws_lambda_example_db::handle_request(ctx.clone(), create_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let login_payload = json!({
        "email": "integration@example.com",
        "password": "secret"
    });
    let login_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::Text(login_payload.to_string()))
        .expect("login request");
    let login_response = aws_lambda_example_db::handle_request(ctx.clone(), login_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(login_response.status(), 200);
    let login_json: serde_json::Value =
        serde_json::from_str(&body_as_string(login_response.body()))?;
    let refresh_token = login_json["refreshToken"]
        .as_str()
        .expect("refresh token")
        .to_string();

    let tokens_after_login = ctx
        .client()
        .query()
        .table_name(ctx.refresh_table())
        .index_name("FamilyUserIndex")
        .key_condition_expression("#fid = :fid AND #uid = :uid")
        .expression_attribute_names("#fid", "familyId")
        .expression_attribute_names("#uid", "userId")
        .expression_attribute_values(
            ":fid",
            aws_sdk_dynamodb::types::AttributeValue::S(family_id.clone()),
        )
        .expression_attribute_values(
            ":uid",
            aws_sdk_dynamodb::types::AttributeValue::S(
                login_json["userId"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            ),
        )
        .projection_expression("refreshToken")
        .send()
        .await?;
    assert_eq!(tokens_after_login.count, 1);

    let refresh_payload = json!({ "refreshToken": refresh_token });
    let refresh_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/token/refresh")
        .header("content-type", "application/json")
        .body(Body::Text(refresh_payload.to_string()))
        .expect("refresh request");
    let refresh_response = aws_lambda_example_db::handle_request(ctx.clone(), refresh_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(refresh_response.status(), 200);
    let refresh_json: serde_json::Value =
        serde_json::from_str(&body_as_string(refresh_response.body()))?;
    let new_refresh_token = refresh_json["refreshToken"]
        .as_str()
        .expect("new refresh")
        .to_string();
    assert_ne!(new_refresh_token, refresh_token);

    let tokens_after_refresh = ctx
        .client()
        .query()
        .table_name(ctx.refresh_table())
        .index_name("FamilyUserIndex")
        .key_condition_expression("#fid = :fid AND #uid = :uid")
        .expression_attribute_names("#fid", "familyId")
        .expression_attribute_names("#uid", "userId")
        .expression_attribute_values(
            ":fid",
            aws_sdk_dynamodb::types::AttributeValue::S(family_id.clone()),
        )
        .expression_attribute_values(
            ":uid",
            aws_sdk_dynamodb::types::AttributeValue::S(
                refresh_json["userId"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            ),
        )
        .projection_expression("refreshToken")
        .send()
        .await?;
    assert_eq!(tokens_after_refresh.count, 1);

    // Old token should be invalid after rotation
    let stale_refresh_payload = json!({ "refreshToken": refresh_token });
    let stale_refresh_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/token/refresh")
        .header("content-type", "application/json")
        .body(Body::Text(stale_refresh_payload.to_string()))
        .expect("stale refresh request");
    let stale_refresh_response =
        aws_lambda_example_db::handle_request(ctx.clone(), stale_refresh_request)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(stale_refresh_response.status(), 401);

    // Revoke new refresh token
    let revoke_payload = json!({ "refreshToken": new_refresh_token.clone() });
    let revoke_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/token/revoke")
        .header("content-type", "application/json")
        .body(Body::Text(revoke_payload.to_string()))
        .expect("revoke request");
    let revoke_response = aws_lambda_example_db::handle_request(ctx.clone(), revoke_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(revoke_response.status(), 200);

    let post_revoke_refresh = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/token/refresh")
        .header("content-type", "application/json")
        .body(Body::Text(
            json!({ "refreshToken": new_refresh_token }).to_string(),
        ))
        .expect("post revoke refresh");
    let post_revoke_response = aws_lambda_example_db::handle_request(ctx, post_revoke_refresh)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(post_revoke_response.status(), 401);

    Ok(())
}
