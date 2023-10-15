use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Collection {
    pub info: CollectionInfo,
    pub item: Vec<CollectionItem>,
    pub auth: Option<CollectionAuth>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionInfo {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionItem {
    pub name: String,
    pub request: CollectionRequest,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionRequest {
    pub method: String,
    pub url: CollectionUrl,
    pub auth: Option<CollectionAuth>,
    pub header: Option<Vec<CollectionRequestHeader>>,
    pub body: Option<RequestBody>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionUrl {
    pub raw: String,
    pub host: Option<Vec<String>>,
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub mode: String,
    pub raw: String,
    pub options: Option<BodyOptions>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BodyOptions {
    pub raw: BodyOptionsRaw,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BodyOptionsRaw {
    language: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionAuth {
    pub r#type: String,
    pub bearer: Option<Vec<AuthValue>>,
    pub oauth2: Option<Vec<AuthValue>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthValue {
    pub key: String,
    pub value: AuthValueUnion,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AuthValueUnion {
    String(String),
    Object (serde_json::Value)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionRequestHeader {
    pub key: String,
    pub value: String,
    pub r#type: String,
}
