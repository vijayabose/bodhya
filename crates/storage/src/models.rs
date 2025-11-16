/// Data models for storage
///
/// This module defines the data structures used for persisting
/// task execution history and quality metrics.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A task execution session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session ended (if completed)
    pub ended_at: Option<DateTime<Utc>>,
    /// User-provided session metadata
    pub metadata: Option<String>,
}

impl Session {
    /// Create a new session with a generated ID
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            ended_at: None,
            metadata: None,
        }
    }

    /// Create a session with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            started_at: Utc::now(),
            ended_at: None,
            metadata: None,
        }
    }

    /// Mark the session as ended
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
    }

    /// Check if the session is still active
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    /// Get session duration in seconds (None if still active)
    pub fn duration_secs(&self) -> Option<i64> {
        self.ended_at
            .map(|end| (end - self.started_at).num_seconds())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// A task execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Unique task ID
    pub id: String,
    /// Session this task belongs to
    pub session_id: String,
    /// Domain of the task (e.g., "code", "mail")
    pub domain: String,
    /// Task description/prompt
    pub description: String,
    /// Agent that handled the task
    pub agent_id: String,
    /// Task status
    pub status: TaskStatus,
    /// When the task started
    pub started_at: DateTime<Utc>,
    /// When the task completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Task result (if successful)
    pub result: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl TaskRecord {
    /// Create a new task record
    pub fn new(
        session_id: impl Into<String>,
        domain: impl Into<String>,
        description: impl Into<String>,
        agent_id: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            domain: domain.into(),
            description: description.into(),
            agent_id: agent_id.into(),
            status: TaskStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            result: None,
            error: None,
        }
    }

    /// Mark the task as successful
    pub fn mark_success(&mut self, result: impl Into<String>) {
        self.status = TaskStatus::Success;
        self.completed_at = Some(Utc::now());
        self.result = Some(result.into());
    }

    /// Mark the task as failed
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error.into());
    }

    /// Get task duration in seconds (None if still running)
    pub fn duration_secs(&self) -> Option<i64> {
        self.completed_at
            .map(|end| (end - self.started_at).num_seconds())
    }
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is currently running
    Running,
    /// Task completed successfully
    Success,
    /// Task failed with an error
    Failed,
}

impl TaskStatus {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Success => "success",
            TaskStatus::Failed => "failed",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "running" => Some(TaskStatus::Running),
            "success" => Some(TaskStatus::Success),
            "failed" => Some(TaskStatus::Failed),
            _ => None,
        }
    }
}

/// Quality metrics for a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Task ID these metrics belong to
    pub task_id: String,
    /// Overall quality score (0-100)
    pub quality_score: Option<f64>,
    /// Number of iterations/refinements
    pub iterations: i32,
    /// Model tokens used (if available)
    pub tokens_used: Option<i64>,
    /// Execution time in milliseconds
    pub execution_time_ms: i64,
    /// Domain-specific metrics (JSON)
    pub custom_metrics: Option<String>,
    /// When metrics were recorded
    pub recorded_at: DateTime<Utc>,
}

impl QualityMetrics {
    /// Create new quality metrics for a task
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            quality_score: None,
            iterations: 0,
            tokens_used: None,
            execution_time_ms: 0,
            custom_metrics: None,
            recorded_at: Utc::now(),
        }
    }

    /// Set quality score
    pub fn with_quality_score(mut self, score: f64) -> Self {
        self.quality_score = Some(score);
        self
    }

    /// Set iterations count
    pub fn with_iterations(mut self, iterations: i32) -> Self {
        self.iterations = iterations;
        self
    }

    /// Set tokens used
    pub fn with_tokens(mut self, tokens: i64) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ms: i64) -> Self {
        self.execution_time_ms = ms;
        self
    }

    /// Set custom metrics as JSON string
    pub fn with_custom_metrics(mut self, metrics: impl Into<String>) -> Self {
        self.custom_metrics = Some(metrics.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert!(!session.id.is_empty());
        assert!(session.is_active());
        assert!(session.ended_at.is_none());
        assert!(session.duration_secs().is_none());
    }

    #[test]
    fn test_session_with_id() {
        let session = Session::with_id("test-session");
        assert_eq!(session.id, "test-session");
        assert!(session.is_active());
    }

    #[test]
    fn test_session_end() {
        let mut session = Session::new();
        assert!(session.is_active());

        session.end();
        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
        assert!(session.duration_secs().is_some());
    }

    #[test]
    fn test_session_default() {
        let session = Session::default();
        assert!(!session.id.is_empty());
        assert!(session.is_active());
    }

    #[test]
    fn test_task_record_creation() {
        let task = TaskRecord::new("session-1", "code", "Write hello world", "code-agent");
        assert!(!task.id.is_empty());
        assert_eq!(task.session_id, "session-1");
        assert_eq!(task.domain, "code");
        assert_eq!(task.description, "Write hello world");
        assert_eq!(task.agent_id, "code-agent");
        assert_eq!(task.status, TaskStatus::Running);
        assert!(task.completed_at.is_none());
        assert!(task.result.is_none());
        assert!(task.error.is_none());
    }

    #[test]
    fn test_task_mark_success() {
        let mut task = TaskRecord::new("session-1", "code", "test", "agent");
        task.mark_success("Task completed!");

        assert_eq!(task.status, TaskStatus::Success);
        assert!(task.completed_at.is_some());
        assert_eq!(task.result, Some("Task completed!".to_string()));
        assert!(task.error.is_none());
        assert!(task.duration_secs().is_some());
    }

    #[test]
    fn test_task_mark_failed() {
        let mut task = TaskRecord::new("session-1", "code", "test", "agent");
        task.mark_failed("Something went wrong");

        assert_eq!(task.status, TaskStatus::Failed);
        assert!(task.completed_at.is_some());
        assert_eq!(task.error, Some("Something went wrong".to_string()));
        assert!(task.result.is_none());
        assert!(task.duration_secs().is_some());
    }

    #[test]
    fn test_task_status_as_str() {
        assert_eq!(TaskStatus::Running.as_str(), "running");
        assert_eq!(TaskStatus::Success.as_str(), "success");
        assert_eq!(TaskStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_task_status_parse() {
        assert_eq!(TaskStatus::parse("running"), Some(TaskStatus::Running));
        assert_eq!(TaskStatus::parse("success"), Some(TaskStatus::Success));
        assert_eq!(TaskStatus::parse("failed"), Some(TaskStatus::Failed));
        assert_eq!(TaskStatus::parse("RUNNING"), Some(TaskStatus::Running));
        assert_eq!(TaskStatus::parse("invalid"), None);
    }

    #[test]
    fn test_quality_metrics_creation() {
        let metrics = QualityMetrics::new("task-1");
        assert_eq!(metrics.task_id, "task-1");
        assert!(metrics.quality_score.is_none());
        assert_eq!(metrics.iterations, 0);
        assert!(metrics.tokens_used.is_none());
        assert_eq!(metrics.execution_time_ms, 0);
        assert!(metrics.custom_metrics.is_none());
    }

    #[test]
    fn test_quality_metrics_builder() {
        let metrics = QualityMetrics::new("task-1")
            .with_quality_score(85.5)
            .with_iterations(3)
            .with_tokens(1500)
            .with_execution_time(2500)
            .with_custom_metrics(r#"{"coverage": 95}"#);

        assert_eq!(metrics.quality_score, Some(85.5));
        assert_eq!(metrics.iterations, 3);
        assert_eq!(metrics.tokens_used, Some(1500));
        assert_eq!(metrics.execution_time_ms, 2500);
        assert!(metrics.custom_metrics.is_some());
    }

    #[test]
    fn test_session_serialization() {
        let session = Session::new();
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, deserialized.id);
    }

    #[test]
    fn test_task_record_serialization() {
        let task = TaskRecord::new("s1", "code", "test", "agent");
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: TaskRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.description, deserialized.description);
    }

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics::new("task-1").with_quality_score(90.0);
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.task_id, deserialized.task_id);
        assert_eq!(metrics.quality_score, deserialized.quality_score);
    }
}
