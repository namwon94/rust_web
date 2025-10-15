-- Add migration script here
CREATE TABLE users(
    email TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    nickname TEXT NOT NULL,
    password_hash TEXT NOT NULL
);