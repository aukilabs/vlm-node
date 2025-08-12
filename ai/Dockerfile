# python-worker/Dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install ollama and pull the models
RUN curl -fsSL https://ollama.com/install.sh | sh
RUN ollama pull llava:7b
RUN ollama pull llama3

COPY requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt

COPY main.py worker.py ./

CMD ollama serve & python main.py
