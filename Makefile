# Makefile

PYTHON=python3
VENV=.venv
ACTIVATE=. $(VENV)/bin/activate

# Default target
.DEFAULT_GOAL := help

.PHONY: server venv install clean worker docker-cpu docker-gpu docker-build-server

help:
	@echo "make venv        - Create virtual environment"
	@echo "make install     - Install dependencies"
	@echo "make server      - Run server (locally)"
	@echo "make docker-cpu  - Build server, worker and ollama and run them in docker"
	@echo "make docker-gpu  - Build server, worker and ollama and run them in docker"
	@echo "make worker      - Run worker (locally)"
	@echo "make clean       - Remove virtual environment, rust releasebuild and cache"

venv:
	@echo "Creating virtual environment..."
	$(PYTHON) -m venv $(VENV)

install: venv
	@echo "Activating venv and installing requirements..."
	@$(ACTIVATE) pip install --upgrade pip && pip install -r worker/requirements.txt
	@echo "Installed requirements"

worker:
	@if [ ! -f .env.local ]; then \
		echo "Error: .env.local file is required but not found"; \
		exit 1; \
	fi
	@make install
	@docker compose up postgres ollama-cpu -d
	@echo "Loading environment variables from .env.local..."
	@set -a; . .env; . .env.local; set +a && . $(VENV)/bin/activate && $(PYTHON) worker/main.py

server:
	@if ! command -v cargo &> /dev/null; then \
		echo "Cargo not found. Installing Rust and Cargo..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		. ${HOME}/.cargo/env; \
	fi
	@if [ ! -f .env.local ]; then \
		echo "Error: .env.local file is required but not found"; \
		exit 1; \
	fi
	@docker compose up postgres ollama-cpu -d
	@sleep 10
	@echo "Loading environment variables from .env.local..."
	@set -a; . .env; . .env.local; set +a && cd server && cargo run

docker-build-server:
	@if ! command -v cargo &> /dev/null; then \
		echo "Cargo not found. Installing Rust and Cargo..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		. ${HOME}/.cargo/env; \
	fi
	@echo "Building server..."
	@cd server && cargo build --release
	@OS_ARCH=$$(docker info --format '{{.OSType}}-{{.Architecture}}' | sed 's/aarch64/arm64/'); \
	echo "OS and ARCH: $$OS_ARCH"; \
	mkdir -p server/target/$$OS_ARCH/release; \
	mv server/target/release/server server/target/$$OS_ARCH/release/server;
	@echo "Server built successfully"
	@echo "Building VLM Node server locally..."
	@docker build -t aukilabs/vlm-node-server:local -f server/Dockerfile server
	@echo "VLM Node server built locally with tag aukilabs/vlm-node-server:local"

docker-cpu:
	@docker compose up ollama-cpu server worker  -d

docker-gpu:
	@docker compose up ollama-gpu server worker -d

clean:
	@echo "Cleaning up virtual environment and pycache..."
	@rm -rf $(VENV) __pycache__ */__pycache__
	@cargo clean

tag: check-version
	@echo "\033[94m\n• Tagging ${VERSION}\033[00m"
	@git tag ${VERSION}
	@git push origin ${VERSION}

check-version:
	@echo "\033[94m\n• Checking Version\033[00m"
ifdef VERSION
	@echo "version set to $(VERSION)"
else
	@echo "\033[91mVERSION is not defined\033[00m"
	@echo "~> make VERSION=\033[90mv0.0.x\033[00m command"
	@exit 1
endif
