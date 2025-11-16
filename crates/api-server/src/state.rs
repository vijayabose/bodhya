/// Application state management
use crate::models::{TaskInfo, TaskResult, TaskStatus};
use bodhya_controller::Controller;
use bodhya_core::{Agent, AgentResult, Task};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Stored task state
#[derive(Debug, Clone)]
pub struct StoredTask {
    pub info: TaskInfo,
    pub result: Option<AgentResult>,
    pub core_task: Task,
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Controller for routing tasks to agents
    pub controller: Arc<Controller>,

    /// Task storage (task_id -> StoredTask)
    pub tasks: Arc<RwLock<HashMap<String, StoredTask>>>,

    /// Server start time
    pub start_time: Instant,
}

impl AppState {
    /// Create new application state
    pub fn new(controller: Controller) -> Self {
        Self {
            controller: Arc::new(controller),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Submit a new task
    pub async fn submit_task(&self, mut task: Task) -> TaskInfo {
        // Ensure task has timestamp
        if task.created_at == DateTime::<Utc>::MIN_UTC {
            task.created_at = Utc::now();
        }

        let info = TaskInfo {
            task_id: task.id.clone(),
            status: TaskStatus::Pending,
            domain: task.domain_hint.clone(),
            description: task.description.clone(),
            created_at: task.created_at,
            started_at: None,
            completed_at: None,
            progress: None,
        };

        let stored = StoredTask {
            info: info.clone(),
            result: None,
            core_task: task,
        };

        self.tasks
            .write()
            .await
            .insert(info.task_id.clone(), stored);

        info
    }

    /// Get task info by ID
    pub async fn get_task_info(&self, task_id: &str) -> Option<TaskInfo> {
        self.tasks.read().await.get(task_id).map(|t| t.info.clone())
    }

    /// Get task result by ID
    pub async fn get_task_result(&self, task_id: &str) -> Option<TaskResult> {
        let tasks = self.tasks.read().await;
        let stored = tasks.get(task_id)?;

        if !stored.info.status.is_terminal() {
            return None;
        }

        stored.result.as_ref().map(|r| TaskResult {
            task_id: task_id.to_string(),
            success: r.success,
            content: if r.success {
                Some(r.content.clone())
            } else {
                None
            },
            error: r.error.clone(),
            metadata: r.metadata.clone(),
            completed_at: stored.info.completed_at.unwrap_or_else(Utc::now),
        })
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        progress: Option<u8>,
    ) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(stored) = tasks.get_mut(task_id) {
            stored.info.status = status;
            stored.info.progress = progress;

            match status {
                TaskStatus::InProgress if stored.info.started_at.is_none() => {
                    stored.info.started_at = Some(Utc::now());
                }
                TaskStatus::Completed | TaskStatus::Failed
                    if stored.info.completed_at.is_none() =>
                {
                    stored.info.completed_at = Some(Utc::now());
                }
                _ => {}
            }

            true
        } else {
            false
        }
    }

    /// Store task result
    pub async fn store_result(&self, task_id: &str, result: AgentResult) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(stored) = tasks.get_mut(task_id) {
            let status = if result.success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };

            stored.info.status = status;
            stored.info.completed_at = Some(Utc::now());
            stored.info.progress = Some(100);
            stored.result = Some(result);

            true
        } else {
            false
        }
    }

    /// Execute a task (blocking operation - should run in background)
    pub async fn execute_task(&self, task_id: &str) -> anyhow::Result<()> {
        // Get the task
        let (task, _info) = {
            let tasks = self.tasks.read().await;
            let stored = tasks
                .get(task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
            (stored.core_task.clone(), stored.info.clone())
        };

        // Update status to in-progress
        self.update_task_status(task_id, TaskStatus::InProgress, Some(0))
            .await;

        // Execute via controller
        let result = self.controller.execute(task).await;

        // Store result
        match result {
            Ok(agent_result) => {
                self.store_result(task_id, agent_result).await;
            }
            Err(e) => {
                let error_result = AgentResult {
                    task_id: task_id.to_string(),
                    content: String::new(),
                    metadata: serde_json::Value::Null,
                    success: false,
                    error: Some(e.to_string()),
                };
                self.store_result(task_id, error_result).await;
            }
        }

        Ok(())
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// List all agents
    pub fn list_agents(&self) -> Vec<Box<dyn Agent>> {
        self.controller.list_agents()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodhya_agent_code::CodeAgent;
    use bodhya_agent_mail::MailAgent;
    use std::sync::Arc;

    fn create_test_controller() -> Controller {
        let code_agent = Arc::new(CodeAgent::new()) as Arc<dyn Agent>;
        let mail_agent = Arc::new(MailAgent::new()) as Arc<dyn Agent>;

        Controller::new(vec![code_agent, mail_agent])
    }

    #[tokio::test]
    async fn test_submit_task() {
        let controller = create_test_controller();
        let state = AppState::new(controller);

        let task = Task::new("test task").with_domain("code");

        let info = state.submit_task(task).await;

        assert_eq!(info.status, TaskStatus::Pending);
        assert_eq!(info.description, "test task");
        assert!(info.started_at.is_none());
    }

    #[tokio::test]
    async fn test_get_task_info() {
        let controller = create_test_controller();
        let state = AppState::new(controller);

        let task = Task::new("test task");
        let submitted = state.submit_task(task).await;

        let retrieved = state.get_task_info(&submitted.task_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().task_id, submitted.task_id);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let controller = create_test_controller();
        let state = AppState::new(controller);

        let task = Task::new("test task");
        let submitted = state.submit_task(task).await;

        let updated = state
            .update_task_status(&submitted.task_id, TaskStatus::InProgress, Some(50))
            .await;
        assert!(updated);

        let info = state.get_task_info(&submitted.task_id).await.unwrap();
        assert_eq!(info.status, TaskStatus::InProgress);
        assert_eq!(info.progress, Some(50));
        assert!(info.started_at.is_some());
    }

    #[tokio::test]
    async fn test_store_result() {
        let controller = create_test_controller();
        let state = AppState::new(controller);

        let task = Task::new("test task");
        let submitted = state.submit_task(task).await;

        let result = AgentResult::success(&submitted.task_id, "test result");
        state.store_result(&submitted.task_id, result).await;

        let info = state.get_task_info(&submitted.task_id).await.unwrap();
        assert_eq!(info.status, TaskStatus::Completed);
        assert_eq!(info.progress, Some(100));

        let task_result = state.get_task_result(&submitted.task_id).await.unwrap();
        assert!(task_result.success);
        assert_eq!(task_result.content, Some("test result".to_string()));
    }

    #[tokio::test]
    async fn test_uptime() {
        let controller = create_test_controller();
        let state = AppState::new(controller);

        // Uptime should be available immediately (0 seconds)
        let uptime1 = state.uptime_seconds();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let uptime2 = state.uptime_seconds();

        // After sleep, uptime should be >= initial uptime
        assert!(uptime2 >= uptime1);
    }
}
