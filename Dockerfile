FROM nvidia/cuda:13.0.0-devel-ubuntu24.04
ARG TARGETPLATFORM TARGETARCH TARGETOS

# Install Python and dependencies for NVIDIA base image
RUN apt-get update && \
      apt-get install -y --no-install-recommends \
        python3-pip \
        ca-certificates \
        libpq-dev \
        git \
        tini \
        && rm -rf /var/lib/apt/lists/* \
        && ln -s /usr/bin/python3.12 /usr/bin/python \
        && ln -s /usr/bin/python3.12 /usr/bin/python3 \
        && ln -s /usr/bin/pip3 /usr/bin/pip;rm -rf /var/lib/apt/lists/*; \

# Install Ollama (models will be pulled dynamically in Python code)
RUN curl -fsSL https://ollama.com/install.sh | sh

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
RUN groupadd -g 101 vlm-node && \
    useradd -m -u 100 -g vlm-node -s /bin/bash vlm-node && \
    chown -R vlm-node:vlm-node /app

USER vlm-node

COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Use tini as entrypoint for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--", "/app/docker-entrypoint.sh"]
