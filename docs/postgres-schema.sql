-- SQL schema for PostgresDagStore
CREATE TABLE IF NOT EXISTS blocks (
    cid TEXT PRIMARY KEY,
    data BYTEA
);
