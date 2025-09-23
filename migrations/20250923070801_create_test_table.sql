-- Add migration script here
-- Add migration script here
CREATE TABLE test_table (
    id uuid NOT NULL,
    name TEXT NOT NULL,
    cntn TEXT,
    created_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL
);