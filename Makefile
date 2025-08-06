# Makefile

PYTHON=python3
VENV=.venv
ACTIVATE=. $(VENV)/bin/activate;

# Default target
.DEFAULT_GOAL := help

help:
	@echo "make venv        - Create virtual environment"
	@echo "make install     - Install dependencies"
	@echo "make run         - Run dispatcher (main.py)"
	@echo "make worker      - Test worker.py independently"
	@echo "make clean       - Remove virtual environment and cache"

venv:
	@echo "Creating virtual environment..."
	$(PYTHON) -m venv $(VENV)

install: venv
	@echo "Activating venv and installing requirements..."
	@$(ACTIVATE) pip install --upgrade pip
	@$(ACTIVATE) pip install -r requirements.txt

run:
	@echo "Starting dispatcher..."
	@docker compose up -d
	@$(ACTIVATE) python main.py

clean:
	@echo "Cleaning up virtual environment and pycache..."
	rm -rf $(VENV) __pycache__ */__pycache__
