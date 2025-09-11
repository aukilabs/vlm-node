FROM debian:bookworm-slim
ARG TARGETPLATFORM TARGETARCH TARGETOS

# Copy Rust server binary and migrations
COPY target/${TARGETOS}-${TARGETARCH}/release/server /app/server
RUN chmod +x /app/server
COPY migrations /app/migrations
ENV MIGRATIONS_PATH=/app/migrations

# Create user and permissions
RUN groupadd -g 101 compute-node && \
    useradd -m -u 100 -g compute-node -s /bin/bash compute-node && \
    chown -R compute-node:compute-node /app

USER compute-node

CMD ["/app/server"]
