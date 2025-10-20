use std::{env, sync::Arc, time::Duration};

use anyhow::Result;
use aws_credential_types::Credentials;
use aws_lambda_example_db::{runtime_env::DeploymentEnv, AppContext};
use aws_sdk_dynamodb::{
    config::Region,
    types::{
        AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType,
        Projection, ProjectionType, ScalarAttributeType,
    },
    Client, Config,
};
use lambda_http::Body;
use uuid::Uuid;

pub fn body_as_string(body: &Body) -> String {
    match body {
        Body::Text(s) => s.clone(),
        Body::Binary(b) => String::from_utf8_lossy(b).to_string(),
        Body::Empty => String::new(),
    }
}

#[allow(dead_code)]
pub struct TestSetup {
    pub ctx: Arc<AppContext>,
    pub client: Client,
    pub user_table: String,
    pub refresh_table: String,
    _guard: TablesGuard,
}

impl Drop for TestSetup {
    fn drop(&mut self) {
        let _ = env::remove_var("ENVIRONMENT_NAME");
        let _ = env::remove_var("CREDENTIALS_TABLE_NAME");
        let _ = env::remove_var("JWT_SECRET");
        let _ = env::remove_var("REFRESH_TOKEN_TABLE_NAME");
    }
}

struct TablesGuard {
    client: Client,
    user_table: String,
    credentials_table: String,
    refresh_table: String,
}

impl TablesGuard {
    async fn new(
        client: Client,
        user_table: String,
        credentials_table: String,
        refresh_table: String,
    ) -> Result<Self> {
        client
            .create_table()
            .table_name(&user_table)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("userId")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("email")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("familyId")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("userName")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("userId")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("EmailIndex")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("email")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("FamilyIdIndex")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("familyId")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("FamilyUserIndex")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("familyId")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("userName")
                            .key_type(KeyType::Range)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()?,
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?;

        client
            .create_table()
            .table_name(&credentials_table)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("email")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("email")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?;

        client
            .create_table()
            .table_name(&refresh_table)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("refreshToken")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("familyId")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("userId")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("refreshToken")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("FamilyIdIndex")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("familyId")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("FamilyUserIndex")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("familyId")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("userId")
                            .key_type(KeyType::Range)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .build()?,
            )
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(Self {
            client,
            user_table,
            credentials_table,
            refresh_table,
        })
    }
}

impl Drop for TablesGuard {
    fn drop(&mut self) {
        let client = self.client.clone();
        let user_table = self.user_table.clone();
        let credentials_table = self.credentials_table.clone();
        let refresh_table = self.refresh_table.clone();
        tokio::spawn(async move {
            let _ = client.delete_table().table_name(&user_table).send().await;
            let _ = client
                .delete_table()
                .table_name(&credentials_table)
                .send()
                .await;
            let _ = client
                .delete_table()
                .table_name(&refresh_table)
                .send()
                .await;
        });
    }
}

pub async fn setup_environment() -> Option<TestSetup> {
    let endpoint =
        env::var("DYNAMODB_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    env::set_var(
        "AWS_ALLOW_HTTP",
        env::var("AWS_ALLOW_HTTP").unwrap_or_else(|_| "true".into()),
    );
    env::set_var(
        "AWS_SDK_LOAD_CONFIG",
        env::var("AWS_SDK_LOAD_CONFIG").unwrap_or_else(|_| "1".into()),
    );

    let region = Region::new(env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()));
    let config = Config::builder()
        .endpoint_url(endpoint)
        .region(region)
        .credentials_provider(Credentials::for_tests())
        .behavior_version_latest()
        .build();
    let client = Client::from_conf(config);

    if client.list_tables().send().await.is_err() {
        eprintln!("skipping integration test: DynamoDB not reachable");
        return None;
    }

    let env_name = format!("IntegrationTest_{}", Uuid::new_v4().simple());
    env::set_var("ENVIRONMENT_NAME", &env_name);
    let user_table = format!("Users_{}", env_name);
    let credentials_table = format!("UserCredentials_{}", env_name);
    env::set_var("CREDENTIALS_TABLE_NAME", &credentials_table);
    let secret_param = format!("/apps/aws-lambda-example-db/{}/JWT_SECRET", env_name);
    env::set_var("JWT_SECRET_PARAMETER", &secret_param);
    env::set_var("JWT_SECRET", "integration-secret");
    let refresh_table = format!("UserRefreshTokens_{}", env_name);
    env::set_var("REFRESH_TOKEN_TABLE_NAME", &refresh_table);

    let guard = TablesGuard::new(
        client.clone(),
        user_table.clone(),
        credentials_table.clone(),
        refresh_table.clone(),
    )
    .await
    .ok()?;

    let _ = DeploymentEnv::detect();
    let ctx = Arc::new(AppContext::new(
        client.clone(),
        user_table.clone(),
        credentials_table.clone(),
        refresh_table.clone(),
        "integration-secret",
    ));

    Some(TestSetup {
        ctx,
        client,
        user_table,
        refresh_table,
        _guard: guard,
    })
}
