-- Add migration script here
CREATE TABLE users(
    user_id uuid PRIMARY KEY,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL
);