/// REST API route handlers
use crate::models::{
    AgentInfo, AgentList, ErrorResponse, HealthResponse, SubmitTaskRequest, SubmitTaskResponse,
    TaskInfo, TaskResult,
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bodhya_core::Task;
use std::sync::Arc;

/// Custom error type for API handlers
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, ErrorResponse::new(msg)),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ErrorResponse::new(msg)),
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorResponse::new(msg))
            }
        };

        (status, Json(error_response)).into_response()
    }
}

/// POST /tasks - Submit a new task
pub async fn submit_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SubmitTaskRequest>,
) -> Result<(StatusCode, Json<SubmitTaskResponse>), ApiError> {
    // Validate request
    if request.description.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Task description cannot be empty".to_string(),
        ));
    }

    // Create core task
    let mut task = Task::new(request.description.clone());

    if let Some(domain) = request.domain {
        task = task.with_domain(domain);
    }

    if request.payload != serde_json::Value::Null {
        task = task.with_payload(request.payload);
    }

    // Submit task
    let task_info = state.submit_task(task.clone()).await;

    // Spawn background execution
    let state_clone = Arc::clone(&state);
    let task_id = task_info.task_id.clone();
    tokio::spawn(async move {
        if let Err(e) = state_clone.execute_task(&task_id).await {
            tracing::error!("Task execution failed: {}", e);
        }
    });

    // Return response
    let response = SubmitTaskResponse {
        task_id: task_info.task_id,
        status: task_info.status,
        created_at: task_info.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /tasks/:id - Get task status
pub async fn get_task_status(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskInfo>, ApiError> {
    let task_info = state
        .get_task_info(&task_id)
        .await
        .ok_or_else(|| ApiError::NotFound(format!("Task {} not found", task_id)))?;

    Ok(Json(task_info))
}

/// GET /tasks/:id/result - Get task result
pub async fn get_task_result(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResult>, ApiError> {
    // Check if task exists
    let task_info = state
        .get_task_info(&task_id)
        .await
        .ok_or_else(|| ApiError::NotFound(format!("Task {} not found", task_id)))?;

    // Check if task is complete
    if !task_info.status.is_terminal() {
        return Err(ApiError::BadRequest(format!(
            "Task {} is not yet complete (status: {:?})",
            task_id, task_info.status
        )));
    }

    // Get result
    let result = state
        .get_task_result(&task_id)
        .await
        .ok_or_else(|| ApiError::InternalError("Result not found".to_string()))?;

    Ok(Json(result))
}

/// GET /agents - List available agents
pub async fn list_agents(State(state): State<Arc<AppState>>) -> Json<AgentList> {
    let agents = state.list_agents();

    let agent_infos: Vec<AgentInfo> = agents
        .iter()
        .map(|agent| {
            let capability = agent.capability();
            AgentInfo {
                id: agent.id().to_string(),
                domain: capability.domain,
                intents: capability.intents,
                description: capability.description,
                enabled: agent.is_enabled(),
            }
        })
        .collect();

    Json(AgentList {
        agents: agent_infos,
    })
}

/// GET /health - Health check
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.uptime_seconds(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;
    use crate::state::AppState;
    use bodhya_agent_code::CodeAgent;
    use bodhya_agent_mail::MailAgent;
    use bodhya_controller::Controller;
    use std::sync::Arc;

    fn create_test_state() -> Arc<AppState> {
        let code_agent = Arc::new(CodeAgent::new()) as Arc<dyn bodhya_core::Agent>;
        let mail_agent = Arc::new(MailAgent::new()) as Arc<dyn bodhya_core::Agent>;
        let controller = Controller::new(vec![code_agent, mail_agent]);

        Arc::new(AppState::new(controller))
    }

    #[tokio::test]
    async fn test_submit_task() {
        let state = create_test_state();

        let request = SubmitTaskRequest {
            domain: Some("code".to_string()),
            description: "test task".to_string(),
            payload: serde_json::json!({}),
        };

        let result = submit_task(State(state), Json(request)).await;
        assert!(result.is_ok());

        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(response.0.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_submit_empty_description() {
        let state = create_test_state();

        let request = SubmitTaskRequest {
            domain: None,
            description: "   ".to_string(),
            payload: serde_json::Value::Null,
        };

        let result = submit_task(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_task_status_not_found() {
        let state = create_test_state();

        let result = get_task_status(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_agents() {
        let state = create_test_state();

        let result = list_agents(State(state)).await;
        assert_eq!(result.0.agents.len(), 2);

        let domains: Vec<_> = result.0.agents.iter().map(|a| a.domain.as_str()).collect();
        assert!(domains.contains(&"code"));
        assert!(domains.contains(&"mail"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state();

        let result = health_check(State(state)).await;
        assert_eq!(result.0.status, "ok");
        assert!(!result.0.version.is_empty());
    }
}
