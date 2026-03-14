CREATE TABLE IF NOT EXISTS request
(
    id          TEXT PRIMARY KEY NOT NULL,
    method      TEXT NOT NULL,
    url         TEXT NOT NULL,
    name        TEXT,
    headers     TEXT,
    body        TEXT
);
