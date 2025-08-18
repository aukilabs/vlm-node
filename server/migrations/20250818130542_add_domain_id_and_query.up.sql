-- Add up migration script here
ALTER TABLE jobs
    ADD COLUMN domain_id TEXT,
    ADD COLUMN query JSONB;
