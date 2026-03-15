# Postie - a Postman alternative

Postie is very much heavily in development but here are some it's early goals:
- Be a free, feature parity (to a basic extent) alternative to Postman.
Mainly started due to a need at work, where Postman cloud accounts are not allowed.
- A basic extent means: making requests, saving request history, and being able to 
save collections and environments.
- Have full interoperability with existing Postman file formats.

## Current State
When there is a new update available there will be a release published that can be downloaded from the releases page.
### Supported
- Native Linux, Mac, and Windows applications (packaging with Cargo Packager)
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
- Infinite levels of collection nesting now supported
- Request history is persisted and previous request/responses can be viewed again
- Manage multiple requests at once with tabs

### Not yet supported
- Tab data persists before hitting submit button on an unsent request
  - currently in order for a tab to persist, the request needs to be submitted or else the tab will be lost
- Exporting saved colellections
- Exporting saved environments
- Deletion of imported collections and environments
- File upload request bodies
- Other Response Types not listed above
- Render XML responses in an interactive way similar to json
- Pre-request scripts (in rust or js)
- Cloud hosting of sqlite tables (very future if at all)

## Building and running
If you wish to run the application source locally, ensure that have a sqlite database set up (see Database Utils) and run the following command:
```shell
cargo run -- postie.sqlite
```

Build and bundling for specific OS targets is now handled via Cargo Packager, configured in Packager.toml.
To build and bundle the application from source, run the following commands:

#### MacOS
```shell
./scripts/bundle_macos.sh
```
#### Linux
```shell
./scripts/bundle_linux.sh
```

#### Windows
```shell
./scripts/bundle_windows.sh
```

## Database Utils

This project uses the rust `sqlx` and `sqlx-cli` packages to manage a SQLite database connection.

To generate a new database file, run the following commands:

* Create a .env file and copy the contents of .env-example into it
* `sqlx db create` - creates the database file, located at the project root
* `sqlx migrate run` - runs all pending migrations

You can then use any SQLite editor to open the `postie.sqlite` file to run queries.
