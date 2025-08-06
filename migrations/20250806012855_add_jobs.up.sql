-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE jobs (
    id TEXT PRIMARY KEY DEFAULT encode(gen_random_bytes(12), 'hex'),
    status VARCHAR(20) NOT NULL DEFAULT 'created',
    job_type VARCHAR(20) NOT NULL,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now(),
    input JSONB NOT NULL,
    output JSONB,
    error JSONB
);
