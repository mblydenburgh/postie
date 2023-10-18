CREATE TABLE IF NOT EXISTS response
(
    id            TEXT PRIMARY KEY NOT NULL,
    status_code   INT NOT NULL,
    name          TEXT,
    headers       TEXT,
    body          TEXT
);
