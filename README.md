# Postie - a Postman alternative

Postie is very much heavily in development but here are some it's early goals:
- Be a free, feature parity (to a basic extent) alternative to Postman.
Mainly started due to a need at work, where Postman cloud accounts are not allowed.
- A basic extent means: making requests, saving request history, and being able to 
save collections and environments.
- Have full interoperability with existing Postman file formats.

## Database Utils

This project uses the rust `sqlx` and `sqlx-cli` packages to manage a SQLite database connection.

To generate a new database file, run the following commands:

* `sqlx db create` - creates the database file, located at `postie.sqlite`

* `sqlx migrate run` - runs all pending migrations

You can then use any SQLite editor to open the `postie.sqlite` file to run queries.
