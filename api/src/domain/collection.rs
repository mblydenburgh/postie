use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Collection {
    pub info: CollectionInfo,
    pub item: Vec<CollectionItemOrFolder>,
    pub auth: Option<CollectionAuth>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionInfo {
    #[serde(rename = "_postman_id")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum CollectionItemOrFolder {
    Item(CollectionItem),
    Folder(CollectionFolder),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionFolder {
    pub name: String,
    pub item: Vec<CollectionItemOrFolder>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionItem {
    pub name: String,
    pub request: CollectionRequest,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionRequest {
    pub method: String,
    pub url: CollectionUrl,
    pub auth: Option<CollectionAuth>,
    pub header: Option<Vec<CollectionRequestHeader>>,
    pub body: Option<RequestBody>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionUrl {
    pub raw: String,
    pub host: Option<Vec<String>>,
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RequestBody {
    pub mode: String,
    pub raw: String,
    pub options: Option<BodyOptions>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct BodyOptions {
    pub raw: BodyOptionsRaw,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct BodyOptionsRaw {
    language: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionAuth {
    pub r#type: String,
    pub bearer: Option<Vec<AuthValue>>,
    pub oauth2: Option<Vec<AuthValue>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct AuthValue {
    pub key: String,
    pub value: AuthValueUnion,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum AuthValueUnion {
    String(String),
    Object(serde_json::Value),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct CollectionRequestHeader {
    pub key: String,
    pub value: String,
    pub r#type: String,
}

// Custom impl for Deserialize to determine wheth parsed thing is a request or sub folder
impl<'de> Deserialize<'de> for CollectionItemOrFolder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

        // Check json structure to determine what enum variant it is
        if let Some(obj) = value.as_object() {
            if obj.contains_key("request") {
                let item: CollectionItem = Deserialize::deserialize(value).unwrap();
                Ok(CollectionItemOrFolder::Item(item))
            } else {
                let item: CollectionFolder = Deserialize::deserialize(value).unwrap();
                Ok(CollectionItemOrFolder::Folder(item))
            }
        } else {
            Err(serde::de::Error::custom(
                "Parsed json doesnt match CollectionItemOrFolder",
            ))
        }
    }
}
