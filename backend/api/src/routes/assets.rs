use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::models::ErrorResponse;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/assets", get(list_assets))
        .route("/assets/{id}", get(get_asset))
}

/// GET /assets — Returns all registered assets.
async fn list_assets(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::db::get_all_assets(&state.pool).await {
        Ok(assets) => (StatusCode::OK, Json(serde_json::to_value(assets).unwrap())),
        Err(e) => {
            tracing::error!("Failed to fetch assets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ErrorResponse {
                    error: "Failed to fetch assets".into(),
                }).unwrap()),
            )
        }
    }
}

/// GET /assets/:id — Returns a single asset by ID.
async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match crate::db::get_asset_by_id(&state.pool, &id).await {
        Ok(Some(asset)) => (StatusCode::OK, Json(serde_json::to_value(asset).unwrap())),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: format!("Asset '{}' not found", id),
            }).unwrap()),
        ),
        Err(e) => {
            tracing::error!("Failed to fetch asset {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::to_value(ErrorResponse {
                    error: "Failed to fetch asset".into(),
                }).unwrap()),
            )
        }
    }
}
