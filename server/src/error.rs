//! Uniform JSON error envelope + JSON-aware extractors.
//!
//! Every failure — handler or extractor — becomes `{"error": "..."}` with an
//! appropriate status, so the frontend never has to parse plain-text axum
//! rejections.
use axum::{
    extract::{rejection::JsonRejection, FromRequest, FromRequestParts, Request},
    http::{header::HeaderValue, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ts_rs::TS;

/// Standardized API error contract per Master Engineering SOP §3.2.
/// Includes `error` alongside `error_code`, `message`, and `details` for backwards compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../app/src/api/types/")]
pub struct ApiErrorResponse {
    pub error_code: String,
    pub message: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub details: Option<serde_json::Value>,
    pub error: String,
}

impl ApiErrorResponse {
    pub fn new(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        let msg = message.into();
        Self {
            error_code: error_code.into(),
            error: msg.clone(),
            message: msg,
            details: None,
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    TooManyRequests(String),
    Internal(String),
}

impl ApiError {
    pub fn not_found(msg: &str) -> Self {
        Self::NotFound(msg.to_string())
    }
    pub fn bad_request(msg: &str) -> Self {
        Self::BadRequest(msg.to_string())
    }
    pub fn unauthorized(msg: &str) -> Self {
        Self::Unauthorized(msg.to_string())
    }
    pub fn forbidden(msg: &str) -> Self {
        Self::Forbidden(msg.to_string())
    }
    pub fn conflict(msg: &str) -> Self {
        Self::Conflict(msg.to_string())
    }
    pub fn too_many_requests(msg: &str) -> Self {
        Self::TooManyRequests(msg.to_string())
    }
    pub fn internal(msg: &str) -> Self {
        Self::Internal(msg.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, msg) = match self {
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", m),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", m),
            ApiError::Forbidden(m) => (StatusCode::FORBIDDEN, "FORBIDDEN", m),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, "CONFLICT", m),
            ApiError::TooManyRequests(m) => (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS", m),
            ApiError::Internal(m) => {
                tracing::error!(error = %m, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "internal server error".to_string(),
                )
            }
        };
        let payload = ApiErrorResponse::new(error_code, msg);
        let mut response = (status, Json(payload)).into_response();
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert("www-authenticate", HeaderValue::from_static("Bearer"));
        }
        response
    }
}

/// `Json<T>` with an `ApiError` rejection, so malformed/oversized bodies still
/// return the JSON error envelope.
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(AppJson(value)),
            Err(rejection) => {
                let status = rejection.status();
                let message = if status == StatusCode::PAYLOAD_TOO_LARGE {
                    "request body is too large".to_string()
                } else {
                    rejection_message(&rejection)
                };
                Err(ApiError::bad_request(&message))
            }
        }
    }
}

/// `Query<T>` with an `ApiError` rejection.
pub struct AppQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for AppQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Query::<T>::from_request_parts(parts, state).await {
            Ok(axum::extract::Query(value)) => Ok(AppQuery(value)),
            Err(rejection) => Err(ApiError::bad_request(&format!(
                "invalid query parameters: {rejection}"
            ))),
        }
    }
}

fn rejection_message(rejection: &JsonRejection) -> String {
    match rejection {
        JsonRejection::JsonDataError(e) => format!("invalid request body: {e}"),
        JsonRejection::JsonSyntaxError(_) => "request body is not valid JSON".to_string(),
        JsonRejection::MissingJsonContentType(_) => {
            "expected Content-Type: application/json".to_string()
        }
        JsonRejection::BytesRejection(_) => "could not read request body".to_string(),
        _ => "invalid request body".to_string(),
    }
}
