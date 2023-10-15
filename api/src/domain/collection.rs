use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Collection {
    info: CollectionInfo,
    item: Vec<CollectionItem>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionInfo {
    name: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionItem {
    name: String,
    request: CollectionRequest
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionRequest {
    method: String,
    url: CollectionUrl
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionUrl {
    raw: String
}
