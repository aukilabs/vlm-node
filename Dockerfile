FROM nvidia/cuda:13.0.0-devel-ubuntu24.04
ARG TARGETPLATFORM TARGETARCH TARGETOS

# Install Python and dependencies for NVIDIA base image
RUN apt-get update && \
      apt-get install -y --no-install-recommends \
        ca-certificates \
        python3 \
        python3-venv \
        libpq-dev \
        git \
        tini \
        curl \
        && rm -rf /var/lib/apt/lists/*;

# Install Ollama (models will be pulled dynamically in Python code)
RUN curl -fsSL https://ollama.com/install.sh | sh
WORKDIR /app

RUN python3 -m venv /app/venv --system-site-packages
ENV PATH="/app/venv/bin:$PATH"

# Copy Python code + requirements, install deps
COPY ai/requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt
COPY ai/*.py .

# Copy Rust server binary and migrations
COPY server/target/${TARGETOS}-${TARGETARCH}/release/server /app/server
RUN chmod +x /app/server
COPY server/migrations /app/migrations
ENV MIGRATIONS_PATH=/app/migrations
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Create user and permissions
RUN groupadd -g 101 compute-node && \
    useradd -m -u 100 -g compute-node -s /bin/bash compute-node && \
    chown -R compute-node:compute-node /app

USER compute-node

# Use tini as entrypoint for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--", "/app/docker-entrypoint.sh"]
