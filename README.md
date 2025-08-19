# Compute Node

`compute-node` is a template project designed to make it easy to spin up new compute nodes that can accept and process jobs in a distributed system. The main goal is to let you focus on the vision computation and AI in the `ai` directory—while the rest of the infrastructure (job queueing, orchestration, database, etc.) is handled for you.

## What does compute node do?

- **Job Management:** Accepts jobs via an API, stores them in a database, and manages their status.
- **Worker Process:** Runs a Python worker that picks up jobs, processes them, and updates their status/results.
- **Input/Output Persistence** Downloads input from a domain server and uploads output to a domain server for later use.
- **Easy Customization:** You only need to modify the logic in `ai/worker.py` to define how jobs are processed.
- **Containerized:** Everything runs in Docker containers, so setup is simple and consistent.

## Prerequisites

Before you begin, make sure you have the following installed on your system:

- [Docker](https://docs.docker.com/get-docker/)

## How to use

1. **Fork this repo.**
2. **Edit `ai/worker.py`** to implement your job processing logic.
3. **(Optional) Edit `ai/main.py`** if you want to customize how the worker runs.
4. **Run `make server`** to start the server, `make worker` to start the worker.
5. **Submit jobs** to the API and your worker will process them.

## Setup

1. **Start the server** (accepts jobs and manages the queue):
Copy `.env` file into `.env.local`, and add your credentials.
```
make server
```

2. **Submit a job** (example using `curl`):
```
curl -X POST http://localhost:8080/jobs \
    -H "Content-Type: application/json" \
    -d '{"job_type":"task_timing_v1","query":{"ids": []},"domain_id":"","input":{"prompt":"","webhook_url":"","vlm_prompt":""}}'
```

3. **Start a worker** (processes jobs from the queue):

```
make worker
```
> You can run multiple workers in parallel if your system resources allow.

4. **Check Job Status**
```
curl "localhost:8080/jobs?limit=100"
curl "localhost:8080/jobs/{job_id}"
```
5. **Upload Job Result**
- Put result to ${DATA_DIR}/output/${job_id}
- Save each result as ${name}.${data_type}
- Call `upload_job_result(conn, job_id, output=None)`


## Folder structure

- `ai/` — Contains the Python worker code. Focus on `worker.py`!
- `server/` — Rust-based API server and job management.
- `docker-compose.yml` — Orchestrates dependencies.
- `Dockerfile` - Dockerfile for building server with worker.
- `Makefile` — Helper commands for setup and running.
