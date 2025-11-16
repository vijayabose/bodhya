/// History command implementation
///
/// This module provides the `bodhya history` command for viewing
/// past task execution history and metrics.
use bodhya_core::Result;
use bodhya_storage::{Session, SqliteStorage, TaskRecord, TaskStatus};
use std::path::PathBuf;

/// Show execution history
pub fn show_history(limit: usize) -> Result<()> {
    let storage = open_storage()?;

    let sessions = storage.list_sessions(limit)?;

    if sessions.is_empty() {
        println!("No task history found.");
        println!("Run some tasks with 'bodhya run' to build history.");
        return Ok(());
    }

    println!("Recent Sessions:");
    println!("{}", "=".repeat(80));

    for session in sessions {
        print_session(&storage, &session)?;
    }

    Ok(())
}

/// Show stats for a specific domain
pub fn show_stats(domain: &str) -> Result<()> {
    let storage = open_storage()?;

    let stats = storage.get_domain_stats(domain)?;

    println!("Domain Statistics: {}", domain);
    println!("{}", "=".repeat(80));
    println!("Total tasks:      {}", stats.total_tasks);
    println!("Successful tasks: {}", stats.successful_tasks);
    println!("Failed tasks:     {}", stats.failed_tasks);
    println!("Success rate:     {:.1}%", stats.success_rate() * 100.0);

    Ok(())
}

/// Print a session and its tasks
fn print_session(storage: &SqliteStorage, session: &Session) -> Result<()> {
    println!(
        "\nSession: {} ({})",
        session.id,
        session.started_at.format("%Y-%m-%d %H:%M:%S")
    );

    if let Some(ended_at) = session.ended_at {
        println!(
            "  Status: Ended at {}",
            ended_at.format("%Y-%m-%d %H:%M:%S")
        );
        if let Some(duration) = session.duration_secs() {
            println!("  Duration: {}s", duration);
        }
    } else {
        println!("  Status: Active");
    }

    let tasks = storage.list_tasks_for_session(&session.id)?;
    if !tasks.is_empty() {
        println!("  Tasks:");
        for task in tasks {
            print_task(&task);

            // Try to get metrics
            if let Ok(Some(metrics)) = storage.get_metrics(&task.id) {
                println!("    Metrics:");
                if let Some(score) = metrics.quality_score {
                    println!("      Quality score: {:.1}/100", score);
                }
                println!("      Iterations: {}", metrics.iterations);
                println!("      Execution time: {}ms", metrics.execution_time_ms);
            }
        }
    } else {
        println!("  No tasks in this session");
    }

    Ok(())
}

/// Print a task record
fn print_task(task: &TaskRecord) {
    let status_icon = match task.status {
        TaskStatus::Success => "✓",
        TaskStatus::Failed => "✗",
        TaskStatus::Running => "⋯",
    };

    println!("    {} [{}] {}", status_icon, task.domain, task.description);
    println!("      Agent: {}", task.agent_id);
    println!("      Started: {}", task.started_at.format("%H:%M:%S"));

    if let Some(completed) = task.completed_at {
        println!("      Completed: {}", completed.format("%H:%M:%S"));
        if let Some(duration) = task.duration_secs() {
            println!("      Duration: {}s", duration);
        }
    }

    if let Some(ref result) = task.result {
        // Truncate long results
        let preview = if result.len() > 100 {
            format!("{}...", &result[..100])
        } else {
            result.clone()
        };
        println!("      Result: {}", preview);
    }

    if let Some(ref error) = task.error {
        println!("      Error: {}", error);
    }
}

/// Open the storage database
fn open_storage() -> Result<SqliteStorage> {
    let db_path = get_db_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    SqliteStorage::new(db_path)
}

/// Get the database path (~/.bodhya/storage/history.db)
fn get_db_path() -> Result<PathBuf> {
    let bodhya_home = crate::utils::bodhya_home()?;
    let storage_dir = bodhya_home.join("storage");
    Ok(storage_dir.join("history.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_history_empty() {
        // With empty storage, should not error
        let result = show_history(10);
        // May fail if storage can't be opened, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_show_stats_empty() {
        // With empty storage, should not error
        let result = show_stats("code");
        // May fail if storage can't be opened, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_get_db_path() {
        let db_path = get_db_path().unwrap();
        assert!(db_path.to_string_lossy().contains("history.db"));
        assert!(db_path.to_string_lossy().contains(".bodhya"));
    }

    #[test]
    fn test_print_task() {
        let task = TaskRecord::new("session-1", "code", "Test task", "code-agent");
        // Should not panic
        print_task(&task);
    }

    #[test]
    fn test_print_task_with_result() {
        let mut task = TaskRecord::new("session-1", "code", "Test task", "code-agent");
        task.mark_success("Task completed successfully!");
        // Should not panic
        print_task(&task);
    }

    #[test]
    fn test_print_task_with_error() {
        let mut task = TaskRecord::new("session-1", "code", "Test task", "code-agent");
        task.mark_failed("Something went wrong");
        // Should not panic
        print_task(&task);
    }

    #[test]
    fn test_print_task_long_result() {
        let mut task = TaskRecord::new("session-1", "code", "Test task", "code-agent");
        let long_result = "a".repeat(200);
        task.mark_success(&long_result);
        // Should not panic and should truncate
        print_task(&task);
    }
}
