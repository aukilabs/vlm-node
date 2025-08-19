-- Add down migration script here
ALTER TABLE jobs
    DROP COLUMN IF EXISTS domain_id,
    DROP COLUMN IF EXISTS query;
