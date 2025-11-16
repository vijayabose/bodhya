/// Bodhya API Server - Main Entry Point
use axum::{
    routing::{get, post},
    Router,
};
use bodhya_agent_code::CodeAgent;
use bodhya_agent_mail::MailAgent;
use bodhya_api_server::{middleware, routes, state::AppState, websocket};
use bodhya_controller::Controller;
use bodhya_core::Agent;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bodhya_api_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Bodhya API Server");

    // Create agents
    let code_agent = Arc::new(CodeAgent::new()) as Arc<dyn Agent>;
    let mail_agent = Arc::new(MailAgent::new()) as Arc<dyn Agent>;

    tracing::info!("Initialized agents: code, mail");

    // Create controller
    let controller = Controller::new(vec![code_agent, mail_agent]);

    // Create application state
    let state = Arc::new(AppState::new(controller));

    // Build router
    let app = Router::new()
        // REST API routes
        .route("/health", get(routes::health_check))
        .route("/agents", get(routes::list_agents))
        .route("/tasks", post(routes::submit_task))
        .route("/tasks/:id", get(routes::get_task_status))
        .route("/tasks/:id/result", get(routes::get_task_result))
        // WebSocket route
        .route("/ws/tasks/:id", get(websocket::ws_handler))
        // Add state and middleware
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::trace_layer())
                .layer(middleware::cors_layer()),
        );

    // Bind address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
