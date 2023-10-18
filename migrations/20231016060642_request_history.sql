CREATE TABLE IF NOT EXISTS request_history
(
    id              TEXT PRIMARY KEY NOT NULL,
    request_id      TEXT NOT NULL,
    response_id     TEXT NOT NULL,
    sent_at         TEXT NOT NULL,
    response        TEXT NOT NULL,

    FOREIGN KEY(request_id)     REFERENCES request(id)
    FOREIGN KEY(response_id)    REFERENCES response(id)
);
