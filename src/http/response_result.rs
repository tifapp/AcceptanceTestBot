use anyhow::Result;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

/// A wrapper that wraps an anyhow::Result into an `IntoResponse` compatible struct.
pub struct ResponseResult<T: IntoResponse> {
    result: Result<T>,
}

impl<T: IntoResponse> ResponseResult<T> {
    pub fn new(result: Result<T>) -> Self {
        Self { result }
    }
}

impl<T: IntoResponse> IntoResponse for ResponseResult<T> {
    fn into_response(self) -> Response {
        match self.result {
            Ok(value) => value.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Error, Ok};
    use reqwest::StatusCode;

    use crate::utils::test_error::TestError;

    use super::*;

    #[test]
    fn into_response_success() {
        let result = ResponseResult::new(Ok((StatusCode::CREATED, "Test")));
        let resp = result.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED)
    }

    #[test]
    fn into_response_failure() {
        let result = ResponseResult::new(Err::<String, _>(Error::new(TestError)));
        let resp = result.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
