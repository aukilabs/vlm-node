ARG TARGETPLATFORM TARGETARCH TARGETOS

FROM nvidia/cuda:13.0.0-devel-ubuntu24.04

# Install Python and dependencies for NVIDIA base image
RUN apt-get update && \
      apt-get install -y --no-install-recommends \
        python3.11 \
        python3.11-dev \
        python3.11-distutils \
        python3-pip \
        ca-certificates \
        curl \
        libpq-dev \
        git \
        && rm -rf /var/lib/apt/lists/* \
        && ln -s /usr/bin/python3.11 /usr/bin/python \
        && ln -s /usr/bin/python3.11 /usr/bin/python3 \
        && ln -s /usr/bin/pip3 /usr/bin/pip;rm -rf /var/lib/apt/lists/*; \

# Install Ollama & pull models
RUN curl -fsSL https://ollama.com/install.sh | sh
RUN ollama serve & sleep 5 && ollama pull llava:7b && ollama pull llama3

WORKDIR /app

# Copy Python code + requirements, install deps
COPY ai/requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt
COPY ai/main.py ai/worker.py ./

# Copy Rust server binary and migrations
COPY server/target/${TARGETOS}-${TARGETARCH}/release/server /app/server
RUN chmod +x /app/server
COPY server/migrations /app/migrations
ENV MIGRATIONS_PATH=/app/migrations

# Create user and permissions
RUN groupadd -g 101 compute-node && \
    useradd -m -u 100 -g compute-node -s /bin/bash compute-node && \
    chown -R compute-node:compute-node /app

USER compute-node

# Run the Rust server as main process, and Python worker in background
CMD ["sh", "-c", "python main.py & exec /app/server"]
