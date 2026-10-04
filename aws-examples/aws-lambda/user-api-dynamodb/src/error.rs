use lambda_http::Error as LambdaError;
use thiserror::Error;
use tracing::error;

/// Internal application errors surfaced during request handling.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("dynamodb error: {0}")]
    Dynamo(String),
    #[error("authentication error: {0}")]
    Auth(String),
}

impl AppError {
    /// Short classification string used for logging.
    pub fn category(&self) -> &'static str {
        match self {
            AppError::Dynamo(_) => "dynamodb",
            AppError::Auth(_) => "auth",
        }
    }
}

/// Convert an internal application error into the Lambda runtime error type.
pub fn lambda_error(err: AppError) -> LambdaError {
    let category = err.category();
    let message = err.to_string();
    error!(category = %category, error = ?err, message = %message, "unhandled application error forwarded to Lambda runtime");
    LambdaError::from(message)
}
