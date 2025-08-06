# FROM rust:1.87.0-slim-bookworm AS sqlx
# RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

FROM debian:bookworm-slim
ARG TARGETPLATFORM TARGETARCH TARGETOS
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN groupadd -g 101 compute-node && \
    useradd -m -u 100 -g compute-node -s /bin/bash compute-node

# COPY --from=sqlx /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY target/${TARGETOS}-${TARGETARCH}/release /app
RUN chmod +x /app/server

COPY migrations /app/migrations
ENV MIGRATIONS_PATH=/app/migrations
WORKDIR /app
USER compute-node

CMD ["/app/server"]
