use std::env;

use ::base64::Engine;
use axum::{
    extract::{Query, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use base64::prelude::BASE64_STANDARD;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::Deserialize;

pub const DEV_RAW_ENDPOINT_PASSWORD: &str = "nah id win";

/// A password to protect an endpoint.
///
/// The endpoints that handle progress updates, or merge/close branches are guarded with a hashed
/// password to prevent bad actors from messing with the data stored by this tool.
#[derive(Debug, Clone)]
pub struct EndpointPassword {
    password: String,
}

impl EndpointPassword {
    pub fn prod() -> Self {
        let base64_hash = env::var("ENDPOINT_HASHED_PASSWORD")
            .expect("Make sure to set the ENDPOINT_HASHED_PASSWORD to a BCrypt Hashed Password in the .env.");
        let bytes = BASE64_STANDARD.decode(base64_hash.as_bytes()).unwrap();
        Self {
            password: String::from_utf8(bytes).unwrap(),
        }
    }

    pub fn dev() -> Self {
        Self {
            password: hash(DEV_RAW_ENDPOINT_PASSWORD, DEFAULT_COST).unwrap(),
        }
    }
}

impl EndpointPassword {
    fn verify(&self, raw_password: &str) -> bool {
        println!("Password: {} Hash: {}", raw_password, self.password);
        verify(raw_password, &self.password).unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
struct QueryParameters {
    password: String,
}

/// Middleware to check if the request has the correct `EndpointPassword`.
pub async fn check_password_middleware(
    req: Request,
    next: Next,
    password: EndpointPassword,
) -> Result<Response, StatusCode> {
    if let Ok::<Query<QueryParameters>, _>(query) = Query::try_from_uri(req.uri()) {
        if password.verify(&query.password) {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[cfg(test)]
mod tests {
    use axum::{middleware::from_fn, response::IntoResponse, routing::post, Router};
    use axum_test::TestServer;

    use super::*;

    #[tokio::test]
    async fn responds_with_forbidden_when_wrong_password() {
        let server = test_server(EndpointPassword::dev());
        let resp = server
            .post("/")
            .add_query_param("password", "djklhnasdjkhdkjhfijkhsdghfkjh")
            .await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn responds_with_forbidden_no_password() {
        let server = test_server(EndpointPassword::dev());
        let resp = server.post("/").await;
        resp.assert_status_forbidden();
    }

    #[tokio::test]
    async fn responds_with_200_when_correct_password() {
        let server = test_server(EndpointPassword::dev());
        let resp = server
            .post("/")
            .add_query_param("password", DEV_RAW_ENDPOINT_PASSWORD)
            .await;
        resp.assert_status_ok();
        resp.assert_text(RESPONSE_STR);
    }

    fn test_server(password: EndpointPassword) -> TestServer {
        let f = from_fn(move |req, next| check_password_middleware(req, next, password.clone()));
        let router = Router::new().route("/", post(endpoint)).route_layer(f);
        TestServer::new(router).unwrap()
    }

    const RESPONSE_STR: &str = "i am the fucking strong";

    async fn endpoint() -> impl IntoResponse {
        RESPONSE_STR
    }
}
