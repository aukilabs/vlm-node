# Task Timing Node

This compute node is a proof of concept (PoC) for using LLaVA (Large Language and Vision Assistant) to analyze image streams of employees performing tasks.


# compute-node

`compute-node` is a template project designed to make it easy to spin up new compute nodes that can accept and process jobs in a distributed system. The main goal is to let you focus on the AI logic—especially in the `ai/worker.py` file—while the rest of the infrastructure (job queueing, orchestration, database, etc.) is handled for you.

## What does compute-node do?

- **Job Management:** Accepts jobs via an API, stores them in a database, and manages their status.
- **Worker Process:** Runs a Python worker that picks up jobs, processes them, and updates their status/results.
- **Easy Customization:** You only need to modify the logic in `ai/worker.py` to define how jobs are processed.
- **Containerized:** Everything runs in Docker containers, so setup is simple and consistent.

## How to use

1. **Clone this repo.**
2. **Edit `ai/worker.py`** to implement your job processing logic.
3. **(Optional) Edit `ai/main.py`** if you want to customize how the worker runs.
4. **Run `make run`** to start the whole stack (API server, database, worker, etc.).
5. **Submit jobs** to the API and your worker will process them.

## Folder structure

- `ai/` — Contains the Python worker code. Focus on `worker.py`!
- `server/` — Rust-based API server and job management.
- `docker-compose.yml` — Orchestrates all services (API, DB, worker).
- `Makefile` — Helper commands for setup and running.

## Why use this template?

- **Separation of concerns:** You focus on the AI/model logic, the template handles infrastructure.
- **Scalable:** Easily spawn more nodes by copying or extending this template.
- **Consistent:** Standardized setup for all compute nodes in your system.

> **Tip:** In most cases, you only need to touch `ai/worker.py` to define how your node processes jobs!
