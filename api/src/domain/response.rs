use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Debug, sqlx::FromRow, sqlx::Encode, sqlx::Decode)]
pub struct DBResponse {
  pub id: String,
  pub status_code: u16,
  pub name: Option<String>,
  #[sqlx(default)]
  pub headers: Vec<ResponseHeader>,
  pub body: Option<String>,
}

#[derive(
  Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug, sqlx::Encode, sqlx::Decode,
)]
pub struct ResponseHeader {
  pub key: String,
  pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OAuthResponse {
  pub access_token: String,
  pub expires_in: i32,
  pub token_type: String,
}

#[derive(Clone, Debug)]
pub struct Response {
  pub status: String,
  pub data: ResponseData,
}

#[derive(Clone, Debug)]
pub enum ResponseData {
  JSON(serde_json::Value),
  TEXT(String),
  XML(String),
  UNKNOWN(String),
}
impl ResponseData {
  /// Returns the raw content as a string without the enum variant wrapper
  pub fn to_raw_string(&self) -> String {
    match self {
      ResponseData::JSON(value) => {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_string())
      }
      ResponseData::TEXT(s) | ResponseData::XML(s) | ResponseData::UNKNOWN(s) => s.clone(),
    }
  }
}
