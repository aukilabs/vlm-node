# VLM Node

VLM Node is designed to process images captured by smart glasses in retail environments. By leveraging Vision Language Models (VLM) and Large Language Models (LLM), it intelligently determines when specific tasks start and end within the store, enabling automated task tracking and analysis.

## Features

- **Vision Language Model Integration**: Uses Ollama for image analysis
- **Job Queue System**: Asynchronous job processing with PostgreSQL backend
- **REST API**: HTTP API for job submission and status tracking
- **Docker Support**: Full containerization with Docker Compose
- **Kubernetes Ready**: Helm charts for production deployment

## Prerequisites

Before you begin, make sure you have the following installed on your system:

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [Ollama](https://ollama.com/download) (for running local language/vision models)

## Quick Start

### Using Docker Compose (Recommended)

1. **Clone the repository**:
```bash
git clone <repository-url>
cd vlm-node
```

2. **Set up environment variables**:
Create a `.env.local` file with your configuration:
```
POSEMESH_EMAIL=
POSEMESH_PASSWORD=
```

3. **Start all services**:
```bash
make docker
```

4. **Verify the setup**:
```bash
# Check if services are running
docker compose ps

# Test the API
curl http://localhost:8080/api/v1/jobs?limit=10
```

### Local Development

#### 1. Start the Database
```bash
docker compose up postgres -d
```

#### 2. Start Ollama
```bash
docker compose up ollama -d
```

#### 3. Start the Server
```bash
make server
```

#### 4. Start the Worker
```bash
make worker
```

## Usage

### Submitting Jobs

Submit a job using the REST API:

```bash
curl -X POST http://localhost:8080/api/v1/jobs \
    -H "Content-Type: application/json" \
    -d '{
        "job_type": "task_timing_v1",
        "query": {"ids": []},
        "domain_id": "",
        "input": {
            "prompt": "Analyze this image for task completion",
            "webhook_url": "",
            "vlm_prompt": "Describe what you see in this image"
        }
    }'
```

### Checking Job Status

```bash
# List all jobs
curl "http://localhost:8080/api/v1/jobs?limit=100"

# Get specific job details
curl "http://localhost:8080/api/v1/jobs/{job_id}"
```

## Realtime Image Inference

You can perform realtime image inference using a WebSocket connection at `ws://localhost:8080/api/v1/ws`.

- **How it works:**
  - Send the image as binary data over the WebSocket.
  - Send the prompt as a text message.
  - The server will respond with inference results.

**Note:**  
If your client is not written in JavaScript, you must also respond to `pong` messages from the server to keep the connection alive.

**Example (JavaScript):**
```javascript
let websocketInstance: WebSocket | null = null;

export function initializeWebSocket(): WebSocket {
    const url = process.env.COMPUTE_NODE_URL;
    if (!url) {
        throw new Error("COMPUTE_NODE_URL environment variable is not set");
    }
    if (websocketInstance) {
        return websocketInstance;
    }
    websocketInstance = new WebSocket(url);
    console.log("WebSocket URL: ", url);

    websocketInstance.onopen = () => {
        console.log("WebSocket connected");
        websocketInstance.send("Describe the art work you see in the photo.");
    }

    let response = "";
    websocketInstance.onmessage = (event) => {
        try {
            let bufferPromise: Promise<ArrayBuffer>;
            if (event.data instanceof ArrayBuffer) {
                bufferPromise = Promise.resolve(event.data);
            } else if (event.data instanceof Blob) {
                bufferPromise = event.data.arrayBuffer();
            } else {
                bufferPromise = Promise.resolve(new TextEncoder().encode(event.data).buffer);
            }

            bufferPromise.then((buffer) => {
                // Try to decode as UTF-8 string
                let text: string;
                try {
                    text = new TextDecoder("utf-8").decode(buffer);
                } catch (e) {
                    console.error("Failed to decode WebSocket binary message as UTF-8", e);
                    return;
                }
                // Try to parse as JSON
                try {
                    const parsed = JSON.parse(text);
                    if (
                        typeof parsed === "object" &&
                        parsed !== null &&
                        typeof parsed.response === "string" &&
                        typeof parsed.done === "boolean"
                    ) {
                        response += parsed.response;
                        if (parsed.done) {
                            console.log("Compute Node response done:", response);
                            response = "";
                        }
                    } else {
                        console.warn("Received message is not in expected format:", parsed);
                    }
                } catch (e) {
                    console.error("Failed to parse WebSocket message as JSON", e, "Raw text:", text);
                }
            });
        } catch (err) {
            console.error("Error handling WebSocket binary message", err);
        }
    }
    websocketInstance.onclose = () => {
        websocketInstance = null;
        console.log("WebSocket closed");
    }
    websocketInstance.onerror = (event) => {
        console.error("WebSocket error", event);
    }
    return websocketInstance;
}

export function sendPhotoToComputeNode(photo: PhotoData): void {
    if (!websocketInstance) {
        initializeWebSocket();
    }
    console.log("[STREAMING] Sending photo to Compute Node", photo.filename);
    websocketInstance.send(photo.buffer);
}
```

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `POSTGRES_URL` | PostgreSQL connection string | - | Yes |
| `VLM_MODEL` | Vision model name for Ollama | `moondream:1.8b` | Yes |
| `LLM_MODEL` | Language model name for Ollama | `llama3:latest` | Yes |
| `OLLAMA_HOST` | Ollama server URL | `http://localhost:11434` | Yes |
| `DATA_DIR` | Directory for storing data | - | Yes |
| `API_URL` | External API URL | - | Yes |
| `DDS_URL` | Data delivery service URL | - | Yes |
| `CLIENT_ID` | Client identifier, any string that helps us identify you | `vlm-node` | Yes |
| `POSEMESH_EMAIL` | Email for external service | - | Yes |
| `POSEMESH_PASSWORD` | Password for external service | - | Yes |
| `IMAGE_BATCH_SIZE` | Number of images to process in batch | `5` | No |

### Model Configuration

The system supports various Ollama models.
https://ollama.com/search, check model input.

To use different models, update the `VLM_MODEL` and `LLM_MODEL` environment variables.

## Deployment

### Using Helm (Kubernetes)

1. **Install the chart**:
```bash
helm install vlm-node ./charts/vlm-node \
    --set postgresql.enabled=true \
    --set security.createSecret=true \
    --set security.posemeshEmail=your_email@example.com \
    --set security.posemeshPassword=your_password
```

2. **Update configuration**:
```bash
helm upgrade vlm-node ./charts/vlm-node \
    --set server.image.tag=v1.0.0
```

### Production Considerations

- Use a managed PostgreSQL database
- Set up proper SSL/TLS certificates
- Configure resource limits and requests
- Set up monitoring and logging
- Use secrets management for sensitive data
- Configure backup strategies

## API Reference

### Jobs Endpoint

- `GET /api/v1/jobs` - List jobs
- `POST /api/v1/jobs` - Create a new job
- `GET /api/v1/jobs/{id}` - Get job details
- `PUT /api/v1/jobs/{id}` - Retry a job

## Troubleshooting

### Common Issues

1. **Ollama model not found**:
   ```bash
   docker exec -it ollama ollama pull moondream:1.8b
   ```

2. **Database connection failed**:
   - Check if PostgreSQL is running
   - Verify connection string in environment variables

3. **Worker not processing jobs**:
   - Check worker logs: `docker compose logs worker`
   - Ensure Ollama is accessible from worker container

4. **UI not loading**:
   - Check if UI container is running: `docker compose ps`
   - Verify port 3000 is not in use

### Logs

View logs for specific services:
```bash
docker compose logs server
docker compose logs worker
docker compose logs ui
docker compose logs postgres
docker compose logs ollama
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

[Add your license information here]

## Support

For issues and questions:
- Create an issue in the repository
- Check the troubleshooting section
- Review the logs for error details
