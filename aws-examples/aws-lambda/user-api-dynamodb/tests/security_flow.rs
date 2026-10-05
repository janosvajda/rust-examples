//! Security tests: each one tries an attack and checks that it fails.
//!
//! Like the other integration tests, these need DynamoDB Local (see the README).

mod common;

use anyhow::Result;
use lambda_http::{Body, Response};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use aws_lambda_example_db::AppContext;
use common::body_as_string;

/// Sends one request to the handler, with an optional bearer token.
async fn call(
    ctx: &Arc<AppContext>,
    method: &str,
    uri: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> Result<Response<Body>> {
    let mut request = lambda_http::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let request = request.body(body.map_or(Body::Empty, |b| Body::Text(b.to_string())))?;
    let request = lambda_http::Request::from(request);
    // `GET /users?userId=…`: the query string must be passed the way API Gateway does
    let request = match uri.split_once("?userId=") {
        Some((_, id)) => lambda_http::RequestExt::with_query_string_parameters(
            request,
            std::collections::HashMap::from([("userId".to_string(), id.to_string())]),
        ),
        None => request,
    };
    aws_lambda_example_db::handle_request(ctx.clone(), request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

fn json_of(response: &Response<Body>) -> Value {
    serde_json::from_str(&body_as_string(response.body())).unwrap_or(Value::Null)
}

/// Registers a user and logs in. Returns (userId, accessToken).
async fn register_and_login(
    ctx: &Arc<AppContext>,
    email: &str,
    password: &str,
) -> Result<(String, String)> {
    let created = call(
        ctx,
        "POST",
        "/users",
        Some(json!({ "userName": email, "email": email, "password": password, "familyId": format!("family-{}", Uuid::new_v4().simple()) })),
        None,
    )
    .await?;
    assert_eq!(created.status(), 201, "{:?}", json_of(&created));
    let login = call(
        ctx,
        "POST",
        "/login",
        Some(json!({ "email": email, "password": password })),
        None,
    )
    .await?;
    assert_eq!(login.status(), 200);
    let login = json_of(&login);
    Ok((
        login["userId"].as_str().unwrap().to_string(),
        login["accessToken"].as_str().unwrap().to_string(),
    ))
}

#[tokio::test]
async fn nobody_can_change_someone_elses_password() -> Result<()> {
    common::with_environment(|setup| Box::pin(async move {
    let ctx = setup.ctx.clone();
    let victim = format!("victim-{}@example.com", Uuid::new_v4().simple());
    let (victim_id, _) = register_and_login(&ctx, &victim, "victim-password").await?;

    // The attack: an "update" that sets a new password for the victim's email.
    let attack = json!({ "userId": "attacker-id", "userName": "x", "email": victim, "password": "attacker-password", "familyId": "f" });
    let without_token = call(&ctx, "POST", "/users", Some(attack.clone()), None).await?;
    assert_eq!(without_token.status(), 401, "an update needs a token");

    // Even with a valid token of ANOTHER user, it must fail.
    let attacker = format!("attacker-{}@example.com", Uuid::new_v4().simple());
    let (_, attacker_token) = register_and_login(&ctx, &attacker, "attacker-password").await?;
    let attack = json!({ "userId": victim_id, "userName": "x", "email": victim, "password": "attacker-password", "familyId": "f" });
    let with_wrong_token = call(&ctx, "POST", "/users", Some(attack), Some(&attacker_token)).await?;
    assert_eq!(with_wrong_token.status(), 403, "a token only allows changing its own user");

    // The victim's password still works, and the attacker's doesn't.
    let ok = call(&ctx, "POST", "/login", Some(json!({ "email": victim, "password": "victim-password" })), None).await?;
    assert_eq!(ok.status(), 200);
    let hacked = call(&ctx, "POST", "/login", Some(json!({ "email": victim, "password": "attacker-password" })), None).await?;
    assert_eq!(hacked.status(), 401);
    Ok(())
    })).await
}

#[tokio::test]
async fn reading_a_user_needs_that_users_token() -> Result<()> {
    common::with_environment(|setup| {
        Box::pin(async move {
            let ctx = setup.ctx.clone();
            let alice = format!("alice-{}@example.com", Uuid::new_v4().simple());
            let bob = format!("bob-{}@example.com", Uuid::new_v4().simple());
            let (alice_id, alice_token) =
                register_and_login(&ctx, &alice, "alice-password").await?;
            let (_, bob_token) = register_and_login(&ctx, &bob, "bob-password").await?;
            let uri = format!("/users?userId={alice_id}");

            assert_eq!(call(&ctx, "GET", &uri, None, None).await?.status(), 401); // no token
            assert_eq!(
                call(&ctx, "GET", &uri, None, Some("not-a-jwt"))
                    .await?
                    .status(),
                401
            ); // garbage
            assert_eq!(
                call(&ctx, "GET", &uri, None, Some(&bob_token))
                    .await?
                    .status(),
                403
            ); // someone else
            let own = call(&ctx, "GET", &uri, None, Some(&alice_token)).await?;
            assert_eq!(own.status(), 200);
            assert_eq!(json_of(&own)["email"], alice);
            Ok(())
        })
    })
    .await
}

#[tokio::test]
async fn weak_or_malformed_input_is_rejected() -> Result<()> {
    common::with_environment(|setup| Box::pin(async move {
    let ctx = setup.ctx.clone();
    let register = |email: &str, password: String| {
        json!({ "userName": "u", "email": email, "password": password, "familyId": "family" })
    };
    for (email, password) in [
        ("short@example.com", String::from("")),
        ("short@example.com", String::from("1234567")),            // 7 characters
        ("huge@example.com", "x".repeat(10_000)),                   // unbounded Argon2 work
        ("not-an-email", String::from("long-enough-password")),
        ("", String::from("long-enough-password")),
    ] {
        let response = call(&ctx, "POST", "/users", Some(register(email, password)), None).await?;
        assert_eq!(response.status(), 400, "{email:?} should be rejected");
    }
    Ok(())
    })).await
}

#[tokio::test]
async fn a_refresh_token_works_only_once_even_at_the_same_moment() -> Result<()> {
    common::with_environment(|setup| Box::pin(async move {
    let ctx = setup.ctx.clone();
    let email = format!("refresh-{}@example.com", Uuid::new_v4().simple());
    call(&ctx, "POST", "/users", Some(json!({ "userName": "r", "email": email, "password": "refresh-password", "familyId": "fam" })), None).await?;
    let login = call(&ctx, "POST", "/login", Some(json!({ "email": email, "password": "refresh-password" })), None).await?;
    let refresh_token = json_of(&login)["refreshToken"].as_str().unwrap().to_string();

    // Two refreshes with the same token at the same time: only one may win.
    let body = json!({ "refreshToken": refresh_token });
    let (first, second) = tokio::join!(
        call(&ctx, "POST", "/token/refresh", Some(body.clone()), None),
        call(&ctx, "POST", "/token/refresh", Some(body), None),
    );
    let statuses = [first?.status().as_u16(), second?.status().as_u16()];
    assert_eq!(statuses.iter().filter(|&&s| s == 200).count(), 1, "statuses: {statuses:?}");
    Ok(())
    })).await
}

#[tokio::test]
async fn concurrent_registration_is_atomic_and_name_updates_keep_uniqueness() -> Result<()> {
    common::with_environment(|setup| Box::pin(async move {
    let ctx = setup.ctx.clone();
    let payload = |email: &str| json!({"userName":"shared", "familyId":"family", "email":email, "password":"original-password"});
    let (a, b) = tokio::join!(call(&ctx, "POST", "/users", Some(payload("a@example.com")), None), call(&ctx, "POST", "/users", Some(payload("b@example.com")), None));
    let responses = [a?, b?];
    assert_eq!(responses.iter().filter(|r| r.status() == 201).count(), 1);
    assert_eq!(responses.iter().filter(|r| r.status() == 409).count(), 1);
    let loser = if responses[0].status() == 409 { "a@example.com" } else { "b@example.com" };
    let login = call(&ctx, "POST", "/login", Some(json!({"email":loser, "password":"original-password"})), None).await?;
    assert_eq!(login.status(), 401, "a failed transaction must not leave credentials");
    let second = call(&ctx, "POST", "/users", Some(json!({"userName":"second", "familyId":"family", "email":loser, "password":"original-password"})), None).await?;
    assert_eq!(second.status(), 201);
    let id = json_of(&second)["userId"].as_str().unwrap().to_owned();
    let login = call(&ctx, "POST", "/login", Some(json!({"email":loser, "password":"original-password"})), None).await?;
    let token = json_of(&login)["accessToken"].as_str().unwrap().to_owned();
    let rename = call(&ctx, "POST", "/users", Some(json!({"userId":id, "userName":"shared", "familyId":"family", "email":loser, "password":"changed-password"})), Some(&token)).await?;
    assert_eq!(rename.status(), 409);
    let login = call(&ctx, "POST", "/login", Some(json!({"email":loser, "password":"original-password"})), None).await?;
    assert_eq!(login.status(), 200, "failed rename must not change the password");
    Ok(())
    })).await
}

#[tokio::test]
async fn concurrent_logins_leave_one_token_and_stale_logout_keeps_it() -> Result<()> {
    common::with_environment(|setup| {
        Box::pin(async move {
            let ctx = setup.ctx.clone();
            register_and_login(&ctx, "parallel@example.com", "parallel-password").await?;
            let body = json!({"email":"parallel@example.com", "password":"parallel-password"});
            let (a, b) = tokio::join!(
                call(&ctx, "POST", "/login", Some(body.clone()), None),
                call(&ctx, "POST", "/login", Some(body), None)
            );
            let tokens = [
                json_of(&a?)["refreshToken"].as_str().unwrap().to_owned(),
                json_of(&b?)["refreshToken"].as_str().unwrap().to_owned(),
            ];
            let user = tokens[0].split_once('.').unwrap().0;
            let row = setup
                .client
                .get_item()
                .table_name(&setup.refresh_table)
                .key(
                    "refreshToken",
                    aws_sdk_dynamodb::types::AttributeValue::S(format!("ACTIVE#{user}")),
                )
                .consistent_read(true)
                .send()
                .await?
                .item
                .unwrap();
            let active = row["token"].as_s().unwrap();
            let stale = tokens.iter().find(|t| *t != active).unwrap();
            call(
                &ctx,
                "POST",
                "/token/revoke",
                Some(json!({"refreshToken":stale})),
                None,
            )
            .await?;
            assert_eq!(
                call(
                    &ctx,
                    "POST",
                    "/token/refresh",
                    Some(json!({"refreshToken":stale})),
                    None
                )
                .await?
                .status(),
                401
            );
            assert_eq!(
                call(
                    &ctx,
                    "POST",
                    "/token/refresh",
                    Some(json!({"refreshToken":active})),
                    None
                )
                .await?
                .status(),
                200
            );
            Ok(())
        })
    })
    .await
}
