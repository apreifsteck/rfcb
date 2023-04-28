-- Add up migration script here
CREATE TABLE votes(
  id SERIAL PRIMARY KEY,
  rfc_id integer REFERENCES request_for_comments NOT NULL,
  deadline TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX vote_rfc_id_fk_index ON votes(rfc_id);

CREATE TABLE motions(
  id SERIAL PRIMARY KEY,
  vote_id integer REFERENCES votes NOT NULL,
  participant_id integer REFERENCES participants NOT NULL,
  type CHARACTER VARYING NOT NULL,
  comment CHARACTER VARYING,

  UNIQUE(vote_id, participant_id)
)
