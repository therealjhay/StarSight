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
        .route("/agents", get(list_agents))
        .route("/agents/{address}/score", get(get_agent_score))
}

/// GET /agents — Returns all agents and their reputation scores.
async fn list_agents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::db::get_all_agent_scores(&state.pool).await {
        Ok(scores) => (StatusCode::OK, Json(serde_json::to_value(scores).unwrap())),
        Err(e) => {
            tracing::error!("Failed to fetch agent scores: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: "Failed to fetch agents".into(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

/// GET /agents/:address/score — Returns the reputation score for a specific agent.
async fn get_agent_score(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    match crate::db::get_agent_score(&state.pool, &address).await {
        Ok(Some(score)) => (StatusCode::OK, Json(serde_json::to_value(score).unwrap())),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ErrorResponse {
                    error: format!("Agent '{}' not found", address),
                })
                .unwrap(),
            ),
        ),
        Err(e) => {
            tracing::error!("Failed to fetch score for agent {}: {}", address, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: "Failed to fetch agent score".into(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}
