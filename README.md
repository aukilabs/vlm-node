This compute node is a proof of concept (PoC) for using LLaVA (Large Language and Vision Assistant) to analyze image streams of employees performing tasks.

# Development

## Prerequisites

Before you begin, make sure you have the following installed on your system:

- [Docker](https://docs.docker.com/get-docker/)
- [Ollama](https://ollama.com/download) (for running local language/vision models)

## Setup

1. **Start the server** (accepts jobs and manages the queue):
Copy `.env` file into `.env.local`, and add your credentials.
```
# Docker
make server-docker
docker compose up server-worker -d
# Or run it locally
make server
```

2. **Submit a job** (example using `curl`):
```

curl -X POST http://localhost:8080/api/v1/jobs \
    -H "Content-Type: application/json" \
    -d '{"job_type":"task_timing_v1","query":{"ids": []},"domain_id":"","input":{"prompt":"","webhook_url":"","vlm_prompt":""}}'
```

3. **Start a worker** (processes jobs from the queue):
> Skip this if you run server from docker compose
It creates a venv and install from requirements.txt.
```
make worker
```

4. **Check Job Status**
```
curl "localhost:8080/api/v1/jobs?limit=100"
curl "localhost:8080/api/v1/jobs/{job_id}"
```

5. **UI**
```
# Docker
docker compose up ui -d
# Or
cd ui
npm install
npm run dev
```
