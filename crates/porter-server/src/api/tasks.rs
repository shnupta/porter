use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use porter_core::models::{CreateTask, Task, UpdateTask};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route(
            "/api/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
}

#[derive(Deserialize)]
struct TaskQuery {
    status: Option<String>,
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskQuery>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = state
        .db
        .list_tasks(query.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    Json(input): Json<CreateTask>,
) -> Result<(StatusCode, Json<Task>), StatusCode> {
    let task = state
        .db
        .create_task(input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.broadcast_task_created(&task);
    Ok((StatusCode::CREATED, Json(task)))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, StatusCode> {
    state
        .db
        .get_task(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateTask>,
) -> Result<Json<Task>, StatusCode> {
    let task = state
        .db
        .update_task(&id, input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    state.broadcast_task_updated(&task);
    Ok(Json(task))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state
        .db
        .delete_task(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        state.broadcast_task_deleted(&id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
