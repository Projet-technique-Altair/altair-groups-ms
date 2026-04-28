/**
 * @file health — service health check endpoint.
 *
 * @remarks
 * Provides a basic endpoint to verify that the service is running and responsive.
 *
 *  - Returns HTTP 200 (OK)
 *  - Includes standard API response format with metadata
 *  - No authentication required
 *
 * Used for:
 *
 *  - Monitoring (uptime checks)
 *  - Container orchestration (readiness/liveness probes)
 *  - Debugging and service validation
 *
 * @packageDocumentation
 */
use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::models::api::{ApiMeta, ApiResponse};

pub async fn health() -> impl IntoResponse {
    let meta = ApiMeta {
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let response = ApiResponse {
        success: true,
        data: None::<()>,
        meta,
    };

    (StatusCode::OK, Json(response))
}
