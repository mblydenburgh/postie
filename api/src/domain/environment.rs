use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EnvironmentFile {
  pub id: String,
  pub name: String,
  pub values: Option<Vec<EnvironmentValue>>,
}

impl Default for EnvironmentFile {
  fn default() -> Self {
    EnvironmentFile {
      id: uuid::Uuid::new_v4().to_string(),
      name: "default".into(),
      values: Some(vec![EnvironmentValue {
        key: "".into(),
        value: "".into(),
        r#type: "default".into(),
        enabled: true,
      }]),
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EnvironmentValue {
  pub key: String,
  pub value: String,
  #[serde(rename = "type")]
  pub r#type: String,
  pub enabled: bool,
}
