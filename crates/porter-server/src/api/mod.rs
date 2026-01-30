mod agents;
mod health;
mod integrations;
mod tasks;

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(tasks::router())
        .merge(agents::router())
        .merge(integrations::router())
}
