pub mod anyhow;
pub mod error;
pub mod result;

pub type ApiResult<T> = Result<T, error::ApiError>;
