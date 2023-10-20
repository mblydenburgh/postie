use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvironmentFile {
    pub id: String,
    pub name: String,
    pub values: Option<Vec<EnvironmentValue>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvironmentValue {
    pub key: String,
    pub value: String,
    pub r#type: String,
    pub enabled: bool
}

