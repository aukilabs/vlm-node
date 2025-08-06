-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE jobs (
    id TEXT PRIMARY KEY DEFAULT encode(gen_random_bytes(12), 'hex'),
    job_status TEXT NOT NULL,
    job_type TEXt NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    input JSONB NOT NULL,
    output JSONB,
    error JSONB
);
