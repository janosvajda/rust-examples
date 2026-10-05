use anyhow::Result;
use aws_lambda_example_db::{AppContext, bootstrap::ensure_tables};
use aws_sdk_dynamodb::{
    Client, Config,
    config::{Credentials, Region},
};
use futures_util::FutureExt;
use lambda_http::Body;
use std::{env, sync::Arc};
use uuid::Uuid;

pub fn body_as_string(body: &Body) -> String {
    match body {
        Body::Text(s) => s.clone(),
        Body::Binary(b) => String::from_utf8(b.clone()).expect("JSON is UTF-8"),
        Body::Empty => String::new(),
    }
}

#[allow(dead_code)]
pub struct TestSetup {
    pub ctx: Arc<AppContext>,
    pub client: Client,
    pub user_table: String,
    pub credentials_table: String,
    pub refresh_table: String,
}
impl TestSetup {
    /// Await cleanup before the test runtime shuts down. Never detach deletion tasks.
    pub async fn cleanup(&self) -> Result<()> {
        let mut first_error = None;
        for table in [
            &self.user_table,
            &self.credentials_table,
            &self.refresh_table,
        ] {
            if let Err(error) = self.client.delete_table().table_name(table).send().await
                && !error
                    .as_service_error()
                    .is_some_and(|e| e.is_resource_not_found_exception())
            {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = first_error {
            return Err(error.into());
        }
        Ok(())
    }
}

pub async fn setup_environment() -> Option<TestSetup> {
    let endpoint = env::var("DYNAMODB_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8000".into());
    let config = Config::builder()
        .endpoint_url(&endpoint)
        .region(Region::new(
            env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into()),
        ))
        .credentials_provider(Credentials::for_tests())
        .behavior_version_latest()
        .build();
    let client = Client::from_conf(config);
    if let Err(error) = client.list_tables().send().await {
        if env::var_os("REQUIRE_DYNAMODB").is_some() {
            panic!("DynamoDB required at {endpoint}: {error}");
        }
        eprintln!("skipping integration test: DynamoDB not reachable at {endpoint}");
        return None;
    }
    let suffix = Uuid::new_v4().simple().to_string();
    let user_table = format!("Users_IntegrationTest_{suffix}");
    let credentials_table = format!("UserCredentials_IntegrationTest_{suffix}");
    let refresh_table = format!("UserRefreshTokens_IntegrationTest_{suffix}");
    let setup = TestSetup {
        ctx: Arc::new(AppContext::new(
            client.clone(),
            user_table.clone(),
            credentials_table.clone(),
            refresh_table.clone(),
            "integration-secret",
        )),
        client,
        user_table,
        credentials_table,
        refresh_table,
    };
    if let Err(error) = ensure_tables(
        &setup.client,
        &setup.user_table,
        &setup.credentials_table,
        &setup.refresh_table,
    )
    .await
    {
        let cleanup = setup.cleanup().await;
        panic!("failed to create integration tables: {error}; cleanup: {cleanup:?}");
    }
    Some(setup)
}

/// Keep the runtime alive for cleanup on success, an error, or an unwinding panic.
pub async fn with_environment(
    test: impl for<'a> FnOnce(
        &'a TestSetup,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>>,
) -> Result<()> {
    let Some(setup) = setup_environment().await else {
        return Ok(());
    };
    let result = std::panic::AssertUnwindSafe(test(&setup))
        .catch_unwind()
        .await;
    let cleanup = setup.cleanup().await;
    match result {
        Ok(Ok(())) => cleanup,
        Ok(Err(error)) => {
            if let Err(e) = cleanup {
                eprintln!("cleanup also failed: {e}");
            }
            Err(error)
        }
        Err(panic) => {
            if let Err(e) = cleanup {
                eprintln!("cleanup after panic failed: {e}");
            }
            std::panic::resume_unwind(panic)
        }
    }
}
