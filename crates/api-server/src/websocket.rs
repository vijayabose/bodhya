/// WebSocket handler for real-time task updates
use crate::models::WsMessage;
use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(task_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, task_id, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, task_id: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Check if task exists
    let task_info = match state.get_task_info(&task_id).await {
        Some(info) => info,
        None => {
            let error = WsMessage::Error {
                message: format!("Task {} not found", task_id),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&error).unwrap()))
                .await;
            return;
        }
    };

    // Send initial status
    let status_msg = WsMessage::TaskStatus {
        task_id: task_info.task_id.clone(),
        status: task_info.status,
        progress: task_info.progress,
    };

    if sender
        .send(Message::Text(serde_json::to_string(&status_msg).unwrap()))
        .await
        .is_err()
    {
        return;
    }

    // If task already complete, send result and close
    if task_info.status.is_terminal() {
        if let Some(result) = state.get_task_result(&task_id).await {
            let complete_msg = WsMessage::TaskComplete {
                task_id: task_id.clone(),
                success: result.success,
                result: result.content,
                error: result.error,
            };

            let _ = sender
                .send(Message::Text(serde_json::to_string(&complete_msg).unwrap()))
                .await;
        }
        return;
    }

    // Poll for updates
    let mut poll_interval = interval(Duration::from_millis(500));
    let mut last_status = task_info.status;
    let mut last_progress = task_info.progress;

    loop {
        tokio::select! {
            // Check for client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle ping/pong
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            if matches!(ws_msg, WsMessage::Ping) {
                                let pong = WsMessage::Pong;
                                if sender
                                    .send(Message::Text(serde_json::to_string(&pong).unwrap()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(_)) => {
                        break;
                    }
                    _ => {}
                }
            }

            // Poll for task updates
            _ = poll_interval.tick() => {
                let current_info = match state.get_task_info(&task_id).await {
                    Some(info) => info,
                    None => break,
                };

                // Send status update if changed
                if current_info.status != last_status || current_info.progress != last_progress {
                    last_status = current_info.status;
                    last_progress = current_info.progress;

                    let status_msg = WsMessage::TaskStatus {
                        task_id: task_id.clone(),
                        status: current_info.status,
                        progress: current_info.progress,
                    };

                    if sender
                        .send(Message::Text(serde_json::to_string(&status_msg).unwrap()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                // If task completed, send result and close
                if current_info.status.is_terminal() {
                    if let Some(result) = state.get_task_result(&task_id).await {
                        let complete_msg = WsMessage::TaskComplete {
                            task_id: task_id.clone(),
                            success: result.success,
                            result: result.content,
                            error: result.error,
                        };

                        let _ = sender
                            .send(Message::Text(
                                serde_json::to_string(&complete_msg).unwrap(),
                            ))
                            .await;
                    }
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::TaskStatus {
            task_id: "test-123".to_string(),
            status: TaskStatus::InProgress,
            progress: Some(75),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("task_status"));
        assert!(json.contains("test-123"));
        assert!(json.contains("in_progress"));
        assert!(json.contains("75"));
    }

    #[test]
    fn test_ws_complete_message() {
        let msg = WsMessage::TaskComplete {
            task_id: "task-456".to_string(),
            success: true,
            result: Some("completed successfully".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("task_complete"));
        assert!(json.contains("task-456"));
        assert!(json.contains("completed successfully"));
    }

    #[test]
    fn test_ws_error_message() {
        let msg = WsMessage::Error {
            message: "Something went wrong".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Something went wrong"));
    }
}
