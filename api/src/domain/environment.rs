use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EnvironmentFile {
  pub id: String,
  pub name: String,
  pub values: Option<Vec<EnvironmentValue>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EnvironmentValue {
  pub key: String,
  pub value: String,
  #[serde(rename = "type")]
  pub r#type: String,
  pub enabled: bool,
}
