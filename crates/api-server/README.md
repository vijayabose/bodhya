# Bodhya API Server

REST and WebSocket API server for Bodhya - Local-first Multi-Agent AI Platform

## Features

- ✅ **REST API** for task submission and management
- ✅ **WebSocket** support for real-time task updates
- ✅ **CORS** enabled for browser clients
- ✅ **Structured logging** with tracing
- ✅ **Agent management** API
- ✅ **Health checks** for monitoring

## Running the Server

### Standalone

```bash
cargo run --bin bodhya-server
```

The server will start on `http://localhost:3000` by default.

### Via CLI

```bash
bodhya serve --port 3000 --host 127.0.0.1
```

## API Endpoints

### Health Check

```bash
GET /health
```

Response:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### List Agents

```bash
GET /agents
```

Response:
```json
{
  "agents": [
    {
      "id": "code",
      "domain": "code",
      "intents": ["generate", "refine", "test"],
      "description": "Code generation agent",
      "enabled": true
    },
    {
      "id": "mail",
      "domain": "mail",
      "intents": ["draft", "refine"],
      "description": "Email drafting agent",
      "enabled": true
    }
  ]
}
```

### Submit Task

```bash
POST /tasks
Content-Type: application/json

{
  "domain": "code",
  "description": "Write a Rust function to calculate factorial",
  "payload": {}
}
```

Response:
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending",
  "created_at": "2025-11-16T12:00:00Z"
}
```

### Get Task Status

```bash
GET /tasks/{task_id}
```

Response:
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "in_progress",
  "domain": "code",
  "description": "Write a Rust function to calculate factorial",
  "created_at": "2025-11-16T12:00:00Z",
  "started_at": "2025-11-16T12:00:01Z",
  "progress": 75
}
```

### Get Task Result

```bash
GET /tasks/{task_id}/result
```

Response:
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true,
  "content": "fn factorial(n: u64) -> u64 { ... }",
  "metadata": {},
  "completed_at": "2025-11-16T12:00:05Z"
}
```

## WebSocket API

Connect to WebSocket for real-time task updates:

```
ws://localhost:3000/ws/tasks/{task_id}
```

### Message Types

#### Task Status Update

```json
{
  "type": "task_status",
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "in_progress",
  "progress": 50
}
```

#### Task Output (Streaming)

```json
{
  "type": "task_output",
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Generated code chunk..."
}
```

#### Task Complete

```json
{
  "type": "task_complete",
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true,
  "result": "Final output content"
}
```

#### Error

```json
{
  "type": "error",
  "message": "Something went wrong"
}
```

#### Ping/Pong (Keep-Alive)

Client sends:
```json
{"type": "ping"}
```

Server responds:
```json
{"type": "pong"}
```

## Example Usage

### cURL Examples

#### Submit a code generation task:

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "domain": "code",
    "description": "Write a hello world program in Rust"
  }'
```

#### Check task status:

```bash
curl http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440000
```

#### Get task result:

```bash
curl http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440000/result
```

### JavaScript/WebSocket Example

```javascript
const taskId = "550e8400-e29b-41d4-a716-446655440000";
const ws = new WebSocket(`ws://localhost:3000/ws/tasks/${taskId}`);

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);

  switch (message.type) {
    case 'task_status':
      console.log(`Status: ${message.status}, Progress: ${message.progress}%`);
      break;
    case 'task_output':
      console.log(`Output: ${message.content}`);
      break;
    case 'task_complete':
      console.log(`Complete! Success: ${message.success}`);
      if (message.result) {
        console.log(`Result: ${message.result}`);
      }
      ws.close();
      break;
    case 'error':
      console.error(`Error: ${message.message}`);
      break;
  }
};

// Send ping to keep connection alive
setInterval(() => {
  ws.send(JSON.stringify({ type: "ping" }));
}, 30000);
```

### Python Example

```python
import requests
import json

# Submit task
response = requests.post('http://localhost:3000/tasks', json={
    'domain': 'mail',
    'description': 'Write a professional email introducing myself'
})
task = response.json()
task_id = task['task_id']

print(f"Task ID: {task_id}")

# Poll for completion
import time
while True:
    status_response = requests.get(f'http://localhost:3000/tasks/{task_id}')
    status = status_response.json()

    print(f"Status: {status['status']}")

    if status['status'] in ['completed', 'failed']:
        break

    time.sleep(1)

# Get result
result_response = requests.get(f'http://localhost:3000/tasks/{task_id}/result')
result = result_response.json()

if result['success']:
    print(f"Result: {result['content']}")
else:
    print(f"Error: {result['error']}")
```

## OpenAPI Documentation

The API is fully documented using OpenAPI 3.0 specification.

View the documentation:
- OpenAPI spec: `openapi.yaml`
- Use [Swagger UI](https://swagger.io/tools/swagger-ui/) or [ReDoc](https://github.com/Redocly/redoc) to visualize

## Configuration

### Environment Variables

- `RUST_LOG` - Logging level (default: `info`)
  - Example: `RUST_LOG=debug cargo run --bin bodhya-server`

### Server Options

- **Host**: `127.0.0.1` (default)
- **Port**: `3000` (default)

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP/WebSocket
       │
┌──────▼──────────────────────┐
│      API Server             │
│  ┌────────────────────────┐ │
│  │ REST Routes            │ │
│  │ - /health              │ │
│  │ - /agents              │ │
│  │ - /tasks               │ │
│  └────────────────────────┘ │
│  ┌────────────────────────┐ │
│  │ WebSocket Handler      │ │
│  └────────────────────────┘ │
│  ┌────────────────────────┐ │
│  │ State Management       │ │
│  │ - Task Storage         │ │
│  │ - Result Cache         │ │
│  └────────────────────────┘ │
└──────┬──────────────────────┘
       │
┌──────▼──────────────────────┐
│   Controller                │
│   (Agent Routing)           │
└──────┬──────────────────────┘
       │
   ┌───┴────┐
   │        │
┌──▼───┐ ┌──▼────┐
│ Code │ │ Mail  │
│Agent │ │ Agent │
└──────┘ └───────┘
```

## Development

### Running Tests

```bash
cargo test --package bodhya-api-server
```

### Building

```bash
cargo build --release --bin bodhya-server
```

### Code Quality

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Error Handling

All errors return appropriate HTTP status codes:

- `200` - Success
- `201` - Created (task submitted)
- `400` - Bad Request (invalid input)
- `404` - Not Found (task doesn't exist)
- `500` - Internal Server Error

Error response format:
```json
{
  "error": "Description of the error",
  "details": "Additional details (optional)"
}
```

## Security Considerations

- **Local-only by default**: Binds to `127.0.0.1`
- **CORS enabled**: Allows cross-origin requests
- **No authentication**: Suitable for local development only
- **Future**: Add authentication/authorization for production deployments

## Performance

- **Async/await**: Built on Tokio for high concurrency
- **Efficient routing**: Minimal overhead with Axum
- **Connection pooling**: WebSocket connections managed efficiently
- **Task polling**: 500ms intervals for status updates

## Troubleshooting

### Port already in use

```
Error: Address already in use (os error 98)
```

Solution: Use a different port:
```bash
bodhya serve --port 3001
```

### Cannot connect to WebSocket

Ensure the task ID exists and the server is running:
```bash
curl http://localhost:3000/tasks/{task_id}
```

## License

MIT OR Apache-2.0
