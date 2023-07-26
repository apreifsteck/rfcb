-- Add up migration script here

CREATE TABLE publishers (
    id SERIAL PRIMARY KEY,
    name CHARACTER VARYING NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL

);
 
CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    publisher_id INTEGER REFERENCES publishers NOT NULL,
    name CHARACTER VARYING NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,

    UNIQUE(name, publisher_id)
);

CREATE INDEX authors_publishers_id_fk_index on authors(publisher_id);
CREATE INDEX authors_name_index ON authors(name);

CREATE TABLE books (
    id SERIAL PRIMARY KEY,
    title CHARACTER VARYING NOT NULL,
    author_id INTEGER REFERENCES authors NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,

    UNIQUE(title, author_id)
);

CREATE INDEX books_authors_id_fk_index on books(author_id);
CREATE INDEX books_title_index on books(title);
