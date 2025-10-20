mod common;

use anyhow::Result;
use aws_lambda_example_db::auth::REFRESH_TOKEN_TTL_SECONDS;
use lambda_http::{self, Body};
use serde_json::json;
use uuid::Uuid;

use common::{body_as_string, setup_environment};

#[tokio::test]
async fn login_success_and_failure() -> Result<()> {
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
    let create_response = aws_lambda_example_db::handle_request(ctx.clone(), create_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(create_response.status(), 201);

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
    assert!(login_json["accessToken"].as_str().is_some());
    assert_eq!(login_json["tokenType"], "Bearer");
    assert!(login_json["refreshToken"].as_str().is_some());
    assert_eq!(login_json["refreshExpiresIn"], REFRESH_TOKEN_TTL_SECONDS);

    let tokens = ctx
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
        .send()
        .await?;
    assert_eq!(tokens.count, 1);

    let bad_login_payload = json!({
        "email": "integration@example.com",
        "password": "wrong"
    });
    let bad_login_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::Text(bad_login_payload.to_string()))
        .expect("bad login request");
    let bad_login_response = aws_lambda_example_db::handle_request(ctx, bad_login_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(bad_login_response.status(), 401);

    Ok(())
}
