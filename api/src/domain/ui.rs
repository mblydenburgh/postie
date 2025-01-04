use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ActiveWindow {
    COLLECTIONS,
    ENVIRONMENT,
    HISTORY,
}
#[derive(Serialize, Deserialize)]
pub enum ImportMode {
    COLLECTION,
    ENVIRONMENT,
}
#[derive(Serialize, Deserialize)]
pub enum NewWindowMode {
    COLLECTION,
    ENVIRONMENT,
    FOLDER,
}
#[derive(Serialize, Deserialize)]
pub enum RequestWindowMode {
    AUTHORIZATION,
    PARAMS,
    HEADERS,
    BODY,
    ENVIRONMENT,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthMode {
    APIKEY,
    BEARER,
    OAUTH2,
    NONE,
}
impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
