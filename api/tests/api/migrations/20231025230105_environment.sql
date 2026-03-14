CREATE TABLE IF NOT EXISTS environment (
   id TEXT PRIMARY KEY NOT NULL,
   name TEXT NOT NULL,
   `values` JSON
);
