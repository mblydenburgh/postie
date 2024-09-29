# Postie - a Postman alternative

Postie is very much heavily in development but here are some it's early goals:
- Be a free, feature parity (to a basic extent) alternative to Postman.
Mainly started due to a need at work, where Postman cloud accounts are not allowed.
- A basic extent means: making requests, saving request history, and being able to 
save collections and environments.
- Have full interoperability with existing Postman file formats.

## Current State
Currently in order to run, Rust must be installed as well as setting up the initial SQLite database (see Database Utils).
Right now there is a fully packaged mac application (sorry Windows, for now) that is able to be built from source.
### Supported
- Submitting GET, POST, PUT, PATCH, DELETE requests
- POST and PUT requests only support application/json body
- Valid handling of responses with Content-Type of application/json, text/html, text/plain
- Environments with variable substition
- Importing postman colellections
- Importing postman environments
- Request history is loading on application start to show previous requests & responses
- Native mac application
- Manage multiple requests at once with tabs

### Not yet supported
- Creating new collections from scratch
- Infinite levels of collection nesting, currently only support one level of folder nesting
- Multiple request tabs open at once
- Request tabs persist between application restarts
- Exporting saved colellections
- Exporting saved environments
- File upload request bodies
- XML request body and response
- Pre-request scripts
- Cloud database and user profiles

## Building and running
To build the application from source, run the following commands:
```shell
cargo build --release
./scripts/bundle.sh
```
This will create a .dmg file in the `target/release` directory that can be run on a mac.

## Database Utils

This project uses the rust `sqlx` and `sqlx-cli` packages to manage a SQLite database connection.

To generate a new database file, run the following commands:

* `sqlx db create` - creates the database file, located at `postie.sqlite`

* `sqlx migrate run` - runs all pending migrations

You can then use any SQLite editor to open the `postie.sqlite` file to run queries.
