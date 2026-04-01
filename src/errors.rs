// ============================================================================
// errors.rs — Unified error handling
//
// Converts all internal errors (ONNX failures, image decode errors, etc.)
// into HTTP-friendly responses via Axum's IntoResponse trait.
// ============================================================================

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("No face detected in image")]
    NoFaceDetected,

    #[error("Image decode failed: {0}")]
    ImageDecode(String),

    #[error("ONNX inference error: {0}")]
    OnnxError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Tensor shape error: {0}")]
    ShapeError(String),
}

// Automatically convert AppError into Axum HTTP responses
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NoFaceDetected  => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::ImageDecode(_)  => (StatusCode::BAD_REQUEST,           self.to_string()),
            AppError::BadRequest(_)   => (StatusCode::BAD_REQUEST,           self.to_string()),
            AppError::OnnxError(_)    => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ShapeError(_)   => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
