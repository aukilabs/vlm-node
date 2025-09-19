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
COPY requirements.txt .
RUN pip install --upgrade pip && pip install -r requirements.txt
COPY *.py .

# Create user and permissions
RUN groupadd -g 101 compute-node && \
    useradd -m -u 100 -g compute-node -s /bin/bash compute-node && \
    chown -R compute-node:compute-node /app

USER compute-node

CMD ["python3", "main.py"]

