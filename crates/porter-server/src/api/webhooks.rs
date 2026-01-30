use crate::AppState;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use porter_core::models::WsEvent;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/webhooks/{integration_id}", post(handle_webhook))
}

async fn handle_webhook(
    State(state): State<AppState>,
    Path(integration_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    let integration = state
        .integration_registry
        .get(&integration_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    let header_map: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|val| (k.as_str().to_string(), val.to_string()))
        })
        .collect();

    let notifications = integration
        .handle_webhook(header_map, body.to_vec())
        .await
        .map_err(|e| {
            tracing::error!(integration = %integration_id, error = %e, "Webhook handler failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Persist and broadcast each notification
    for notification in &notifications {
        if let Err(e) = state
            .db
            .create_notification(
                &notification.notification_type,
                &notification.message,
                notification.integration_id.as_deref(),
            )
            .await
        {
            tracing::error!(error = %e, "Failed to persist webhook notification");
        }
        let _ = state.ws_tx.send(WsEvent::Notification(notification.clone()));
    }

    Ok(StatusCode::OK)
}
