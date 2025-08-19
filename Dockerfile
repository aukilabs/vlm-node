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

WORKDIR /app

RUN python3 -m venv /app/venv --system-site-packages
ENV PATH="/app/venv/bin:$PATH"

# Copy Python code + requirements, install deps
COPY ai/requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt
COPY ai/main.py ai/worker.py ./

# Copy Rust server binary and migrations
COPY server/target/${TARGETOS}-${TARGETARCH}/release/server /app/server
RUN chmod +x /app/server
COPY server/migrations /app/migrations
ENV MIGRATIONS_PATH=/app/migrations
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# Create user and permissions
RUN groupadd -g 101 vlm-node && \
    useradd -m -u 100 -g vlm-node -s /bin/bash vlm-node && \
    chown -R vlm-node:vlm-node /app

USER vlm-node

# Use tini as entrypoint for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--", "/app/docker-entrypoint.sh"]
