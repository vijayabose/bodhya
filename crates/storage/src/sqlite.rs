/// SQLite storage backend
///
/// This module provides SQLite-based persistence for task execution
/// history and quality metrics.
use crate::models::{QualityMetrics, Session, TaskRecord, TaskStatus};
use bodhya_core::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// SQLite storage manager
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Create a new storage manager with a database at the given path
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref())
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to open database: {}", e)))?;

        let storage = Self { conn };
        storage.initialize_schema()?;
        Ok(storage)
    }

    /// Create an in-memory storage (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            bodhya_core::Error::Io(format!("Failed to create in-memory database: {}", e))
        })?;

        let storage = Self { conn };
        storage.initialize_schema()?;
        Ok(storage)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        // Sessions table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    started_at TEXT NOT NULL,
                    ended_at TEXT,
                    metadata TEXT
                )",
                [],
            )
            .map_err(|e| {
                bodhya_core::Error::Io(format!("Failed to create sessions table: {}", e))
            })?;

        // Tasks table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    domain TEXT NOT NULL,
                    description TEXT NOT NULL,
                    agent_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    completed_at TEXT,
                    result TEXT,
                    error TEXT,
                    FOREIGN KEY (session_id) REFERENCES sessions(id)
                )",
                [],
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to create tasks table: {}", e)))?;

        // Quality metrics table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS quality_metrics (
                    task_id TEXT PRIMARY KEY,
                    quality_score REAL,
                    iterations INTEGER NOT NULL,
                    tokens_used INTEGER,
                    execution_time_ms INTEGER NOT NULL,
                    custom_metrics TEXT,
                    recorded_at TEXT NOT NULL,
                    FOREIGN KEY (task_id) REFERENCES tasks(id)
                )",
                [],
            )
            .map_err(|e| {
                bodhya_core::Error::Io(format!("Failed to create quality_metrics table: {}", e))
            })?;

        // Create indexes for common queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_tasks_session ON tasks(session_id)",
                [],
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to create index: {}", e)))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_tasks_domain ON tasks(domain)",
                [],
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Save a session
    pub fn save_session(&self, session: &Session) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO sessions (id, started_at, ended_at, metadata)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    &session.id,
                    session.started_at.to_rfc3339(),
                    session.ended_at.as_ref().map(|dt| dt.to_rfc3339()),
                    &session.metadata,
                ],
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to save session: {}", e)))?;

        Ok(())
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, started_at, ended_at, metadata FROM sessions WHERE id = ?1")
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to prepare query: {}", e)))?;

        let session = stmt
            .query_row(params![session_id], |row| {
                let started_str: String = row.get(1)?;
                let ended_str: Option<String> = row.get(2)?;

                Ok(Session {
                    id: row.get(0)?,
                    started_at: chrono::DateTime::parse_from_rfc3339(&started_str)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    ended_at: ended_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    metadata: row.get(3)?,
                })
            })
            .optional()
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to query session: {}", e)))?;

        Ok(session)
    }

    /// List all sessions (most recent first)
    pub fn list_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, started_at, ended_at, metadata FROM sessions
                 ORDER BY started_at DESC LIMIT ?1",
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to prepare query: {}", e)))?;

        let sessions = stmt
            .query_map(params![limit], |row| {
                let started_str: String = row.get(1)?;
                let ended_str: Option<String> = row.get(2)?;

                Ok(Session {
                    id: row.get(0)?,
                    started_at: chrono::DateTime::parse_from_rfc3339(&started_str)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    ended_at: ended_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    metadata: row.get(3)?,
                })
            })
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to query sessions: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to collect sessions: {}", e)))?;

        Ok(sessions)
    }

    /// Save a task record
    pub fn save_task(&self, task: &TaskRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO tasks
                 (id, session_id, domain, description, agent_id, status,
                  started_at, completed_at, result, error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    &task.id,
                    &task.session_id,
                    &task.domain,
                    &task.description,
                    &task.agent_id,
                    task.status.as_str(),
                    task.started_at.to_rfc3339(),
                    task.completed_at.as_ref().map(|dt| dt.to_rfc3339()),
                    &task.result,
                    &task.error,
                ],
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to save task: {}", e)))?;

        Ok(())
    }

    /// Get a task by ID
    pub fn get_task(&self, task_id: &str) -> Result<Option<TaskRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, domain, description, agent_id, status,
                        started_at, completed_at, result, error
                 FROM tasks WHERE id = ?1",
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to prepare query: {}", e)))?;

        let task = stmt
            .query_row(params![task_id], |row| {
                let started_str: String = row.get(6)?;
                let completed_str: Option<String> = row.get(7)?;
                let status_str: String = row.get(5)?;

                Ok(TaskRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    domain: row.get(2)?,
                    description: row.get(3)?,
                    agent_id: row.get(4)?,
                    status: TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Failed),
                    started_at: chrono::DateTime::parse_from_rfc3339(&started_str)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    completed_at: completed_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    result: row.get(8)?,
                    error: row.get(9)?,
                })
            })
            .optional()
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to query task: {}", e)))?;

        Ok(task)
    }

    /// List tasks for a session
    pub fn list_tasks_for_session(&self, session_id: &str) -> Result<Vec<TaskRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, domain, description, agent_id, status,
                        started_at, completed_at, result, error
                 FROM tasks WHERE session_id = ?1
                 ORDER BY started_at ASC",
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to prepare query: {}", e)))?;

        let tasks = stmt
            .query_map(params![session_id], |row| {
                let started_str: String = row.get(6)?;
                let completed_str: Option<String> = row.get(7)?;
                let status_str: String = row.get(5)?;

                Ok(TaskRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    domain: row.get(2)?,
                    description: row.get(3)?,
                    agent_id: row.get(4)?,
                    status: TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Failed),
                    started_at: chrono::DateTime::parse_from_rfc3339(&started_str)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    completed_at: completed_str.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    result: row.get(8)?,
                    error: row.get(9)?,
                })
            })
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to query tasks: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to collect tasks: {}", e)))?;

        Ok(tasks)
    }

    /// Save quality metrics
    pub fn save_metrics(&self, metrics: &QualityMetrics) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO quality_metrics
                 (task_id, quality_score, iterations, tokens_used,
                  execution_time_ms, custom_metrics, recorded_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    &metrics.task_id,
                    metrics.quality_score,
                    metrics.iterations,
                    metrics.tokens_used,
                    metrics.execution_time_ms,
                    &metrics.custom_metrics,
                    metrics.recorded_at.to_rfc3339(),
                ],
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to save metrics: {}", e)))?;

        Ok(())
    }

    /// Get quality metrics for a task
    pub fn get_metrics(&self, task_id: &str) -> Result<Option<QualityMetrics>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT task_id, quality_score, iterations, tokens_used,
                        execution_time_ms, custom_metrics, recorded_at
                 FROM quality_metrics WHERE task_id = ?1",
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to prepare query: {}", e)))?;

        let metrics = stmt
            .query_row(params![task_id], |row| {
                let recorded_str: String = row.get(6)?;

                Ok(QualityMetrics {
                    task_id: row.get(0)?,
                    quality_score: row.get(1)?,
                    iterations: row.get(2)?,
                    tokens_used: row.get(3)?,
                    execution_time_ms: row.get(4)?,
                    custom_metrics: row.get(5)?,
                    recorded_at: chrono::DateTime::parse_from_rfc3339(&recorded_str)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                })
            })
            .optional()
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to query metrics: {}", e)))?;

        Ok(metrics)
    }

    /// Get aggregate statistics by domain
    pub fn get_domain_stats(&self, domain: &str) -> Result<DomainStats> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COUNT(*),
                        SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END)
                 FROM tasks WHERE domain = ?1",
            )
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to prepare query: {}", e)))?;

        let stats = stmt
            .query_row(params![domain], |row| {
                Ok(DomainStats {
                    domain: domain.to_string(),
                    total_tasks: row.get(0)?,
                    successful_tasks: row.get(1)?,
                    failed_tasks: row.get(2)?,
                })
            })
            .map_err(|e| bodhya_core::Error::Io(format!("Failed to query stats: {}", e)))?;

        Ok(stats)
    }
}

/// Statistics for a specific domain
#[derive(Debug, Clone)]
pub struct DomainStats {
    pub domain: String,
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
}

impl DomainStats {
    /// Calculate success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.successful_tasks as f64 / self.total_tasks as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = SqliteStorage::in_memory().unwrap();
        // Storage created successfully
        assert!(storage.conn.is_autocommit());
    }

    #[test]
    fn test_save_and_get_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session = Session::new();

        storage.save_session(&session).unwrap();
        let retrieved = storage.get_session(&session.id).unwrap();

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, session.id);
        assert!(retrieved.is_active());
    }

    #[test]
    fn test_save_and_get_task() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session = Session::new();
        storage.save_session(&session).unwrap();

        let task = TaskRecord::new(&session.id, "code", "Write tests", "code-agent");
        storage.save_task(&task).unwrap();

        let retrieved = storage.get_task(&task.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, task.id);
        assert_eq!(retrieved.description, "Write tests");
        assert_eq!(retrieved.domain, "code");
    }

    #[test]
    fn test_list_sessions() {
        let storage = SqliteStorage::in_memory().unwrap();

        let session1 = Session::new();
        let session2 = Session::new();
        storage.save_session(&session1).unwrap();
        storage.save_session(&session2).unwrap();

        let sessions = storage.list_sessions(10).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_list_tasks_for_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session = Session::new();
        storage.save_session(&session).unwrap();

        let task1 = TaskRecord::new(&session.id, "code", "Task 1", "code-agent");
        let task2 = TaskRecord::new(&session.id, "code", "Task 2", "code-agent");
        storage.save_task(&task1).unwrap();
        storage.save_task(&task2).unwrap();

        let tasks = storage.list_tasks_for_session(&session.id).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_save_and_get_metrics() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session = Session::new();
        storage.save_session(&session).unwrap();

        let task = TaskRecord::new(&session.id, "code", "Test", "code-agent");
        storage.save_task(&task).unwrap();

        let metrics = QualityMetrics::new(&task.id)
            .with_quality_score(85.5)
            .with_iterations(3)
            .with_execution_time(1500);

        storage.save_metrics(&metrics).unwrap();

        let retrieved = storage.get_metrics(&task.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.quality_score, Some(85.5));
        assert_eq!(retrieved.iterations, 3);
    }

    #[test]
    fn test_domain_stats() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session = Session::new();
        storage.save_session(&session).unwrap();

        let mut task1 = TaskRecord::new(&session.id, "code", "Task 1", "code-agent");
        task1.mark_success("Done");
        storage.save_task(&task1).unwrap();

        let mut task2 = TaskRecord::new(&session.id, "code", "Task 2", "code-agent");
        task2.mark_failed("Error");
        storage.save_task(&task2).unwrap();

        let stats = storage.get_domain_stats("code").unwrap();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.successful_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.success_rate(), 0.5);
    }

    #[test]
    fn test_domain_stats_success_rate() {
        let stats = DomainStats {
            domain: "code".to_string(),
            total_tasks: 10,
            successful_tasks: 8,
            failed_tasks: 2,
        };

        assert_eq!(stats.success_rate(), 0.8);
    }

    #[test]
    fn test_domain_stats_empty() {
        let stats = DomainStats {
            domain: "code".to_string(),
            total_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
        };

        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_update_session() {
        let storage = SqliteStorage::in_memory().unwrap();
        let mut session = Session::new();
        storage.save_session(&session).unwrap();

        // Update session to mark as ended
        session.end();
        storage.save_session(&session).unwrap();

        let retrieved = storage.get_session(&session.id).unwrap().unwrap();
        assert!(!retrieved.is_active());
        assert!(retrieved.ended_at.is_some());
    }

    #[test]
    fn test_update_task() {
        let storage = SqliteStorage::in_memory().unwrap();
        let session = Session::new();
        storage.save_session(&session).unwrap();

        let mut task = TaskRecord::new(&session.id, "code", "Test", "code-agent");
        storage.save_task(&task).unwrap();

        // Update task to mark as successful
        task.mark_success("Task completed successfully");
        storage.save_task(&task).unwrap();

        let retrieved = storage.get_task(&task.id).unwrap().unwrap();
        assert_eq!(retrieved.status, TaskStatus::Success);
        assert!(retrieved.result.is_some());
    }
}
