//! Application-scoped context shared across request handlers.

use aws_sdk_dynamodb::Client;

/// Holds shared clients and configuration (DynamoDB tables plus JWT secret).
#[derive(Clone)]
pub struct AppContext {
    client: Client,
    table_name: String,
    credentials_table: String,
    refresh_table: String,
    jwt_secret: String,
}

impl AppContext {
    /// Construct a new context for the given DynamoDB client and target tables.
    pub fn new(
        client: Client,
        table_name: impl Into<String>,
        credentials_table: impl Into<String>,
        refresh_table: impl Into<String>,
        jwt_secret: impl Into<String>,
    ) -> Self {
        Self {
            client,
            table_name: table_name.into(),
            credentials_table: credentials_table.into(),
            refresh_table: refresh_table.into(),
            jwt_secret: jwt_secret.into(),
        }
    }

    /// Borrow the underlying DynamoDB client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Name of the DynamoDB table the handler should operate on.
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Credentials table name (stores email/password hashes).
    pub fn credentials_table(&self) -> &str {
        &self.credentials_table
    }

    /// Refresh token table name.
    pub fn refresh_table(&self) -> &str {
        &self.refresh_table
    }

    /// Symmetric signing secret used for issuing JWTs.
    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}
