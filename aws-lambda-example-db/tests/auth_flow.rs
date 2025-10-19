mod common;

use anyhow::Result;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use lambda_http::{self, Body};
use serde_json::json;
use uuid::Uuid;

use common::{body_as_string, setup_environment};

#[tokio::test]
async fn jwt_contains_expected_claims() -> Result<()> {
    let Some(setup) = setup_environment().await else {
        return Ok(());
    };
    let ctx = setup.ctx.clone();

    let family_id = format!("family-{}", Uuid::new_v4().simple());
    let create_payload = json!({
        "userName": "integration-user",
        "email": "integration@example.com",
        "password": "secret",
        "familyId": family_id.clone()
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
    let login_response = aws_lambda_example_db::handle_request(ctx, login_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(login_response.status(), 200);
    let login_json: serde_json::Value =
        serde_json::from_str(&body_as_string(login_response.body()))?;
    let token = login_json["accessToken"].as_str().expect("token");

    let decoded = decode::<serde_json::Value>(
        token,
        &DecodingKey::from_secret("integration-secret".as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    assert_eq!(decoded.claims["fid"], family_id);
    assert!(decode::<serde_json::Value>(
        token,
        &DecodingKey::from_secret("wrong-secret".as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .is_err());

    Ok(())
}
