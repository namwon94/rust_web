-- Add migration script here
ALTER TABLE users ADD COLUMN created_at timestamptz NOT NULL;
ALTER TABLE users ADD COLUMN updated_at timestamptz NOT NULL;