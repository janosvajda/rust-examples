//! Lambda entrypoint.
//!
//! The binary initialises logging, discovers which environment it is running in,
//! bootstraps dependencies (DynamoDB tables locally, SSM secrets everywhere), and
//! then hands execution to `lambda_http`. Each invocation reuses the `AppContext`
//! so the SDK clients and configuration are cached across requests.

use std::sync::Arc;

use aws_lambda_example_db::{
    bootstrap::ensure_tables, handle_request, runtime_env::DeploymentEnv, AppContext,
};
use aws_sdk_dynamodb::Client;
use lambda_http::{run, service_fn, Error as LambdaError};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .json()
        .with_current_span(false)
        .init();

    let environment = DeploymentEnv::detect();
    let table_name = environment.table_name();
    info!(
        environment = environment.name(),
        %table_name,
        resolution = %environment.source(),
        "initialising Lambda runtime"
    );

    let credentials_table = std::env::var("CREDENTIALS_TABLE_NAME")
        .map_err(|_| LambdaError::from("missing CREDENTIALS_TABLE_NAME env var"))?;
    let refresh_table = std::env::var("REFRESH_TOKEN_TABLE_NAME")
        .map_err(|_| LambdaError::from("missing REFRESH_TOKEN_TABLE_NAME env var"))?;
    let jwt_secret_param = std::env::var("JWT_SECRET_PARAMETER")
        .map_err(|_| LambdaError::from("missing JWT_SECRET_PARAMETER env var"))?;

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    let bootstrap_tables = std::env::var("BOOTSTRAP_DYNAMODB_TABLES")
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            _ => false,
        })
        .unwrap_or_else(|_| environment.name().eq_ignore_ascii_case("Local"));

    if bootstrap_tables {
        ensure_tables(&client, &table_name, &credentials_table, &refresh_table)
            .await
            .map_err(|e| LambdaError::from(format!("failed to ensure DynamoDB tables: {e}")))?;
    } else {
        info!(
            environment = environment.name(),
            "skipping DynamoDB table bootstrap"
        );
    }
    let ssm = aws_sdk_ssm::Client::new(&config);
    let jwt_secret = match ssm
        .get_parameter()
        .name(&jwt_secret_param)
        .with_decryption(true)
        .send()
        .await
    {
        Ok(resp) => resp
            .parameter
            .and_then(|p| p.value)
            .ok_or_else(|| LambdaError::from("JWT secret parameter missing value"))?,
        Err(err) => {
            warn!(
                "failed to fetch JWT secret from SSM ({}); falling back to JWT_SECRET env var",
                err
            );
            std::env::var("JWT_SECRET").map_err(|_| {
                LambdaError::from("missing JWT_SECRET env var fallback after SSM lookup failure")
            })?
        }
    };

    let ctx = Arc::new(AppContext::new(
        client,
        table_name,
        credentials_table,
        refresh_table,
        jwt_secret,
    ));

    run(service_fn(move |event| {
        let ctx = ctx.clone();
        async move { handle_request(ctx, event).await }
    }))
    .await
}
