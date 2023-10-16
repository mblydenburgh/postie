CREATE TABLE IF NOT EXISTS response
(
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT,
    headers     TEXT,
    body        TEXT
);
