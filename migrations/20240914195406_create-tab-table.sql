CREATE TABLE IF NOT EXISTS tabs (
    id          TEXT PRIMARY KEY NOT NULL,
    method      TEXT NOT NULL,
    url         TEXT NOT NULL,
    req_headers     TEXT,
    req_body        TEXT,
    res_status TEXT,
    res_body TEXT,
    res_headers TEXT
);

INSERT INTO tabs (id, method, url, req_headers, req_body, res_status, res_body, res_headers)
  VALUES ('b4884455-c04d-4ee7-a713-b1b21e706246', 'GET', 'https://httpbin.org/json', NULL, NULL, NULL, NULL, NULL);
