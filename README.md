# Postie - a Postman alternative

Postie is very much heavily in development but here are some it's early goals:
- Be a free, feature parity (to a basic extent) alternative to Postman.
Mainly started due to a need at work, where Postman cloud accounts are not allowed.
- A basic extent means: making requests, saving request history, and being able to 
save collections and environments.
- Have full interoperability with existing Postman file formats.

## Current State
When there is a new update available there will be a release published that can be downloaded from the releases page.
Note: currently when you update a version of the app, all data will be lost since a fresh db is packaged in each release. 
This will be improved in the future but if you wish to persist your saved data, first save a backup copy of the .sqlite file 
in your currently used .app.
### Supported
- Native mac application
- Submitting GET, POST, PUT, PATCH, DELETE requests
  - POST and PUT requests only support application/json body
- Response Types:
  - application/json
  - application/xml (rendered as plain text)
  - text/html
  - text/plain
  - text/xml (rendered as plain text)
- Authentication types:
  - Bearer Token
  - OAuth 2.0
  - API Key (via header)
  - Unauthenticated
- Environments with variable substition
- Importing postman colellections
- Importing postman environments
- Creating new collections from scratch
- Saving requests to existing collections
- Request history is persisted and previous request/responses can be viewed again
- Manage multiple requests at once with tabs

### Not yet supported
- Infinite levels of collection nesting, currently only support one level of folder nesting
- Tab data persists before hitting submit button on an unsent request
  - currently in order for a tab to persist, the request needs to be submitted or else the tab will be lost
- Exporting saved colellections
- Exporting saved environments
- Deletion of imported collections and environments
- Deletion of collection folders
- File upload request bodies
- Other Response Types not listed above
- Render XML responses in an interactive way similar to json
- Pre-request scripts (in rust or js)
- Cloud hosting of sqlite tables (very future if at all)

## Building and running
If you wish to run the application source locallyh, ensure that have a sqlite database set up (see Database Utils) and run the following command:
```shell
cargo run DATABASE_URL=postie.sqlite
```

To build and bundle the application from source, run the following commands:
```shell
cargo build --release
./scripts/bundle.sh
```
This will create a .dmg file in the `target/release` directory that can be run on a mac using the version specified in the bundle script.
Note: a fresh database is bundled with each new app so all data will be lost when updating. For now you can restore request history by
overriding the sqlite file that in packaged in the final `.app`.

## Database Utils

This project uses the rust `sqlx` and `sqlx-cli` packages to manage a SQLite database connection.

To generate a new database file, run the following commands:

* Create a .env file and copy the contents of .env-example into it
* `sqlx db create` - creates the database file, located at `postie.sqlite`
* `sqlx migrate run` - runs all pending migrations

You can then use any SQLite editor to open the `postie.sqlite` file to run queries.

