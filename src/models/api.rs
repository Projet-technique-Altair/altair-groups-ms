/**
 * @file api — standard API response and error models for Groups service.
 *
 * @remarks
 * This module defines the unified response structure used across the Groups microservice.
 * It ensures consistency between all HTTP responses by enforcing a shared format for:
 *
 *  - Successful responses (`ApiResponse<T>`)
 *  - Error responses (`ApiErrorResponse`)
 *  - Metadata (`ApiMeta`)
 *
 * Key design principles:
 *
 *  - Every response includes a `success` flag for quick status checks
 *  - All responses embed metadata (`request_id`, `timestamp`) for observability and tracing
 *  - Errors follow a structured format with a machine-readable `code` and human-readable `message`
 *  - Optional `details` field allows attaching additional debugging context when needed
 *
 * This structure is used by the API gateway and frontend to:
 *
 *  - Standardize error handling
 *  - Enable consistent logging and debugging
 *  - Simplify client-side data parsing
 *
 * Usage:
 *
 *  - `ApiResponse::success(data)` for successful operations
 *  - `ApiErrorResponse` for failures
 *
 * @packageDocumentation
 */

use serde::Serialize;

#[derive(Serialize)]
pub struct ApiMeta {
    pub request_id: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub error: ApiError,
    pub meta: ApiMeta,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub meta: ApiMeta,
}

impl ApiMeta {
    pub fn new() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: ApiMeta::new(),
        }
    }
}
