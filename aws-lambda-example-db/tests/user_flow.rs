mod common;

use std::collections::HashMap;

use anyhow::Result;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::{self, Body, RequestExt};
use serde_json::json;
use uuid::Uuid;

use common::{body_as_string, setup_environment};

#[tokio::test]
async fn user_crud_and_constraints_flow() -> Result<()> {
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
    let create_response = aws_lambda_example_db::handle_request(ctx.clone(), create_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(create_response.status(), 201);
    let created_body = body_as_string(create_response.body());
    let created_json: serde_json::Value = serde_json::from_str(&created_body)?;
    let user_id = created_json["userId"]
        .as_str()
        .expect("user id")
        .to_string();

    let fetch_request = lambda_http::http::Request::builder()
        .method("GET")
        .uri("/users")
        .body(Body::Empty)
        .expect("fetch request")
        .with_query_string_parameters(
            [("userId".to_string(), user_id.clone())]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );
    let fetch_response = aws_lambda_example_db::handle_request(ctx.clone(), fetch_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(fetch_response.status(), 200);
    let fetched_json: serde_json::Value =
        serde_json::from_str(&body_as_string(fetch_response.body()))?;
    assert_eq!(fetched_json["userId"], user_id);
    assert_eq!(fetched_json["userName"], "integration-user");
    assert_eq!(fetched_json["familyId"], family_id);

    let update_payload = json!({
        "userId": user_id,
        "userName": "integration-user-updated",
        "email": "integration@example.com",
        "password": "secret",
        "familyId": family_id.clone()
    });
    let update_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::Text(update_payload.to_string()))
        .expect("update request");
    let update_response = aws_lambda_example_db::handle_request(ctx.clone(), update_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(update_response.status(), 201);

    let fetch_updated = lambda_http::http::Request::builder()
        .method("GET")
        .uri("/users")
        .body(Body::Empty)
        .expect("fetch updated")
        .with_query_string_parameters(
            [("userId".to_string(), user_id.clone())]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );
    let updated_response = aws_lambda_example_db::handle_request(ctx.clone(), fetch_updated)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(updated_response.status(), 200);
    let updated_json: serde_json::Value =
        serde_json::from_str(&body_as_string(updated_response.body()))?;
    assert_eq!(updated_json["userName"], "integration-user-updated");
    assert_eq!(updated_json["familyId"], family_id);

    let duplicate_payload = json!({
        "userName": "integration-user-updated",
        "email": "duplicate@example.com",
        "password": "dup",
        "familyId": family_id.clone()
    });
    let duplicate_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::Text(duplicate_payload.to_string()))
        .expect("duplicate request");
    let duplicate_response = aws_lambda_example_db::handle_request(ctx.clone(), duplicate_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(duplicate_response.status(), 409);

    let second_user_payload = json!({
        "userName": "integration-user-second",
        "email": "second@example.com",
        "password": "secret2",
        "familyId": family_id.clone()
    });
    let second_request = lambda_http::http::Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::Text(second_user_payload.to_string()))
        .expect("second request");
    let second_response = aws_lambda_example_db::handle_request(ctx.clone(), second_request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    assert_eq!(second_response.status(), 201);

    let query_resp = setup
        .client
        .query()
        .table_name(&setup.user_table)
        .index_name("FamilyUserIndex")
        .key_condition_expression("#fid = :fid")
        .expression_attribute_names("#fid", "familyId")
        .expression_attribute_values(":fid", AttributeValue::S(family_id.clone()))
        .send()
        .await?;
    assert_eq!(query_resp.count(), 2);

    Ok(())
}
