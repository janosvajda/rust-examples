use lambda_http::Error as LambdaError;
use thiserror::Error;

/// Internal application errors surfaced during request handling.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("dynamodb error: {0}")]
    Dynamo(String),
    #[error("authentication error: {0}")]
    Auth(String),
}

/// Convert an internal application error into the Lambda runtime error type.
pub fn lambda_error(err: AppError) -> LambdaError {
    LambdaError::from(err.to_string())
}
