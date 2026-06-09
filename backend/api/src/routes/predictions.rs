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
        .route("/predictions", get(list_predictions))
        .route("/predictions/{id}", get(get_prediction))
        .route(
            "/predictions/agent/{address}",
            get(get_predictions_by_agent),
        )
}

/// GET /predictions — Returns all predictions.
async fn list_predictions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::db::get_all_predictions(&state.pool).await {
        Ok(predictions) => (
            StatusCode::OK,
            Json(serde_json::to_value(predictions).unwrap()),
        ),
        Err(e) => {
            tracing::error!("Failed to fetch predictions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: "Failed to fetch predictions".into(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

/// GET /predictions/:id — Returns a single prediction by ID.
async fn get_prediction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match crate::db::get_prediction_by_id(&state.pool, id).await {
        Ok(Some(prediction)) => (
            StatusCode::OK,
            Json(serde_json::to_value(prediction).unwrap()),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ErrorResponse {
                    error: format!("Prediction {} not found", id),
                })
                .unwrap(),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to fetch prediction {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: "Failed to fetch prediction".into(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

/// GET /predictions/agent/:address — Returns all predictions by a specific agent.
async fn get_predictions_by_agent(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    match crate::db::get_predictions_by_agent(&state.pool, &address).await {
        Ok(predictions) => (
            StatusCode::OK,
            Json(serde_json::to_value(predictions).unwrap()),
        ),
        Err(e) => {
            tracing::error!("Failed to fetch predictions for agent {}: {}", address, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: "Failed to fetch predictions for agent".into(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}
