-- Add up migration script here


CREATE TABLE IF NOT EXISTS users (
  id         SERIAL NOT NULL PRIMARY KEY,
  username   TEXT NOT NULL UNIQUE,
  password   TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_permissions (
    user_id  INTEGER NOT NULL,
    token    TEXT NOT NULL
);