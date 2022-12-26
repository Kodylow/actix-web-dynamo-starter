use crate::model::task::Task;
use crate::repo::ddb::DDBRepository;

use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    post, put,
    web::{Data, Json, Path},
    HttpResponse,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Deserilize, Serialize)]
pub struct TaskIdentifier {
    task_global_id: String,
}

#[derive(Deserialize)]
pub struct TaskCompletionRequest {
    result_file: String,
}

#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    task_type: String,
    source_file: String,
}

pub enum TaskError {
    TaskNotFound,
    TaskUpdateFailure,
    TaskCreationFailure,
    BadTaskRequest,
}

impl RequestError for TaskError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TaskError::TaskNotFound => StatusCode::NOT_FOUND,
            TaskError::TaskUpdateFailure => StatusCode::FAILED_DEPENDENCY,
            TaskError::TaskCreationFailure => StatusCode::FAILED_DEPENDENCY,
            TaskError::BadTaskRequest => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/api/task/{task_global_id")]
pub async fn get_task(
    task_identifier: Path<TaskIdentifier>,
    ddb_repo: Data<DDBRepository>,
) -> Result<Json<Task>, TaskError> {
    let task = ddb_repo
        .get_task(task_identifier.into_inner().task_global_id)
        .await;

    match task {
        Some(task) => return Ok(Json(task)),
        None => Err(TaskError::TaskNotFound),
    }
}

#[post(/api/task/)]
pub async fn submit_task(
    ddb_repo: Data<DDBRepository>,
    request: Json<SubmitTaskRequest>
) -> Result<Json<TaskIdentifier>, TaskError> {
    let task = Task::new(
        request.user_id.clone(),
        request.task_type.clone(),
        request.source_file.clone(),
    );

    let task_identifier = task.get_global_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier { task_global_id: task_identifier })),
        Err(err) => Err(TaskError::BadTaskRequest)
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>,
    task_global_id: String,
    result_file: Option<String>
) -> Result<Json<TaskIdentifier>, TaskError> {
    let mut task = match ddb_repo.get_task(task_global_id).await {
        Some(task) => task,
        None => Err(TaskError::TaskNotFound)
    };
    
    if !task.can_transition_to(&new_state) {
        Err(TaskError::BadTaskRequest)
    }

    task.state = new_state;;
    task.result_file = result_file;

    let task_identifier = task.get_global_id();
    match ddb_rep.put_task(task).await {
        OK(()) => Ok(Json(TaskIdentifier {task_global_id: task_identifier})),
        Err(_) => Err(TaskError::TaskUpdateFailure),
    }
}

#[put("/api/task/{task_global_id}/start")]
pub async fn start_task(
    ddb_repo: Data<DDBRepository>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    ddb_repo,
    task_identifier.into_inner().task_global_id,

}