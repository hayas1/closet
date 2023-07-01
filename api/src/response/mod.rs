pub mod error;
pub mod result;

pub type ApiResult<T> = Result<result::ApiResponse<T>, error::ApiError>;
