use aws_sdk_dynamodb::{
    types::{
        AttributeDefinition, BillingMode, GlobalSecondaryIndex, KeySchemaElement, KeyType,
        Projection, ProjectionType, ScalarAttributeType, TableStatus,
    },
    Client,
};
use tokio::time::{sleep, Duration};

pub async fn ensure_tables(
    client: &Client,
    user_table: &str,
    credentials_table: &str,
    refresh_table: &str,
) -> Result<(), aws_sdk_dynamodb::Error> {
    ensure_user_table(client, user_table).await?;
    ensure_credentials_table(client, credentials_table).await?;
    ensure_refresh_table(client, refresh_table).await?;
    Ok(())
}

async fn ensure_user_table(client: &Client, table: &str) -> Result<(), aws_sdk_dynamodb::Error> {
    if table_exists(client, table).await? {
        return Ok(());
    }

    client
        .create_table()
        .table_name(table)
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("userId")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("static userId definition"),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("email")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("static email definition"),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("familyId")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("static familyId definition"),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("userName")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("static userName definition"),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("userId")
                .key_type(KeyType::Hash)
                .build()
                .expect("static userId key"),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("EmailIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("email")
                        .key_type(KeyType::Hash)
                        .build()
                        .expect("static email key"),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::All)
                        .build(),
                )
                .build()
                .expect("EmailIndex definition"),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("FamilyIdIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("familyId")
                        .key_type(KeyType::Hash)
                        .build()
                        .expect("static familyId key"),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::All)
                        .build(),
                )
                .build()
                .expect("FamilyIdIndex definition"),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("FamilyUserIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("familyId")
                        .key_type(KeyType::Hash)
                        .build()
                        .expect("familyId key"),
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("userName")
                        .key_type(KeyType::Range)
                        .build()
                        .expect("userName range key"),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::All)
                        .build(),
                )
                .build()
                .expect("FamilyUserIndex definition"),
        )
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    wait_for_active(client, table).await
}

async fn ensure_credentials_table(
    client: &Client,
    table: &str,
) -> Result<(), aws_sdk_dynamodb::Error> {
    if table_exists(client, table).await? {
        return Ok(());
    }

    client
        .create_table()
        .table_name(table)
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("email")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("credentials email definition"),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("email")
                .key_type(KeyType::Hash)
                .build()
                .expect("credentials email key"),
        )
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    wait_for_active(client, table).await
}

async fn ensure_refresh_table(client: &Client, table: &str) -> Result<(), aws_sdk_dynamodb::Error> {
    if table_exists(client, table).await? {
        return Ok(());
    }

    client
        .create_table()
        .table_name(table)
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("refreshToken")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("refresh token definition"),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("familyId")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("refresh familyId definition"),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("userId")
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("refresh userId definition"),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("refreshToken")
                .key_type(KeyType::Hash)
                .build()
                .expect("refresh token key"),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("FamilyIdIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("familyId")
                        .key_type(KeyType::Hash)
                        .build()
                        .expect("refresh familyId key"),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::All)
                        .build(),
                )
                .build()
                .expect("FamilyIdIndex definition"),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("FamilyUserIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("familyId")
                        .key_type(KeyType::Hash)
                        .build()
                        .expect("refresh familyId key"),
                )
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("userId")
                        .key_type(KeyType::Range)
                        .build()
                        .expect("refresh userId range key"),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::All)
                        .build(),
                )
                .build()
                .expect("FamilyUserIndex definition"),
        )
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await?;

    wait_for_active(client, table).await
}

async fn table_exists(client: &Client, table: &str) -> Result<bool, aws_sdk_dynamodb::Error> {
    let mut last_evaluated = None;
    loop {
        let mut req = client.list_tables();
        if let Some(ref start) = last_evaluated {
            req = req.exclusive_start_table_name(start);
        }
        let resp = req.send().await?;
        if resp
            .table_names
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .any(|name| name == table)
        {
            return Ok(true);
        }
        if let Some(next) = resp.last_evaluated_table_name {
            last_evaluated = Some(next);
        } else {
            break;
        }
    }
    Ok(false)
}

async fn wait_for_active(client: &Client, table: &str) -> Result<(), aws_sdk_dynamodb::Error> {
    for _ in 0..20 {
        let resp = client.describe_table().table_name(table).send().await?;
        if resp
            .table
            .and_then(|t| t.table_status)
            .map_or(false, |status| status == TableStatus::Active)
        {
            return Ok(());
        }
        sleep(Duration::from_millis(200)).await;
    }
    Ok(())
}
