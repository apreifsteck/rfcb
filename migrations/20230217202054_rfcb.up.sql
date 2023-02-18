CREATE TABLE request_for_comments (
  id SERIAL PRIMARY KEY,
  proposal CHARACTER VARYING NOT NULL,
  topic CHARACTER VARYING NOT NULL,
  status CHARACTER VARYING NOT NULL DEFAULT 'active',
  supersedes INTEGER REFERENCES request_for_comments ON DELETE RESTRICT,
  superseded_by INTEGER REFERENCES request_for_comments ON DELETE RESTRICT,

  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX request_for_comments_status_index ON request_for_comments(status);

