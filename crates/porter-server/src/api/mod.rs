mod health;
mod tasks;
mod agents;
mod skills;

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(tasks::router())
        .merge(agents::router())
        .merge(skills::router())
}
