use std::sync::Arc;

use axum::Router;

pub mod agents;
pub mod assets;
pub mod predictions;
pub mod ws;

use crate::AppState;

/// Builds the combined application router from all sub-route modules.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(assets::router())
        .merge(predictions::router())
        .merge(agents::router())
        .merge(ws::router())
}
