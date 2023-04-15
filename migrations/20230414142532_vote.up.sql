-- Add up migration script here
CREATE TABLE votes(
  id SERIAL PRIMARY KEY,
  request_for_comment_id integer REFERENCES request_for_comments NOT NULL,
  deadline TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX vote_request_for_comments_fk_index ON votes(request_for_comment_id);

CREATE TABLE motions(
  id SERIAL PRIMARY KEY,
  vote_id integer REFERENCES votes NOT NULL,
  participant_id integer REFERENCES participants NOT NULL,
  type CHARACTER VARYING NOT NULL,
  comment CHARACTER VARYING,

  UNIQUE(vote_id, participant_id)
)
