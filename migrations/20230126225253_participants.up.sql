-- Add up migration script here
CREATE TABLE participants (
  id SERIAL PRIMARY KEY,
  username CHARACTER VARYING NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
)
