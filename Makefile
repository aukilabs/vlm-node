# Makefile

PYTHON=python3
VENV=.venv
ACTIVATE=. $(VENV)/bin/activate;

# Default target
.DEFAULT_GOAL := help

.PHONY: server venv install clean

help:
	@echo "make venv        - Create virtual environment"
	@echo "make install     - Install dependencies"
	@echo "make server      - Run server"
	@echo "make clean       - Remove virtual environment and cache"

venv:
	@echo "Creating virtual environment..."
	$(PYTHON) -m venv $(VENV)

install: venv
	@echo "Activating venv and installing requirements..."
	@$(ACTIVATE) pip install --upgrade pip
	@$(ACTIVATE) pip install -r requirements.txt

worker:
	@if [ ! -f .env.local ]; then \
		echo "Error: .env.local file is required but not found"; \
		exit 1; \
	fi
	@echo "Loading environment variables from .env.local..."
	@set -a; source .env; source .env.local; set +a && cd ai && $(PYTHON) worker.py

server:
	@if ! command -v cargo &> /dev/null; then \
		echo "Cargo not found. Installing Rust and Cargo..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		source ~/.cargo/env; \
	fi
	@if [ ! -f .env.local ]; then \
		echo "Error: .env.local file is required but not found"; \
		exit 1; \
	fi
	@docker compose up postgres -d
	@sleep 10
	@echo "Loading environment variables from .env.local..."
	@set -a; source .env; source .env.local; set +a && cd server && cargo run

server-docker:
	@docker compose up postgres server -d

build-server:
	@echo "Building for TARGETPLATFORM: $${TARGETPLATFORM}, CLIENT: $${CLIENT}"
	@cd server && ../build-rust.sh

clean:
	@echo "Cleaning up virtual environment and pycache..."
	rm -rf $(VENV) __pycache__ */__pycache__
