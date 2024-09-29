use serde::{Deserialize, Serialize};
use sqlx;

#[derive(Debug, sqlx::FromRow, sqlx::Encode, sqlx::Decode)]
pub struct DBRequest {
    pub id: String,
    pub method: String,
    pub url: String,
    pub name: Option<String>,
    #[sqlx(default)]
    pub headers: Vec<RequestHeader>,
    pub body: Option<String>,
}

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug, sqlx::Encode, sqlx::Decode,
)]
pub struct RequestHeader {
    pub key: String,
    pub value: String,
}
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug, sqlx::Encode, sqlx::Decode,
)]
pub struct RequestHeaders(pub Vec<RequestHeader>);
impl FromIterator<(String, String)> for RequestHeaders {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut h = RequestHeaders(Vec::new());
        for (k, v) in iter {
            h.0.push(RequestHeader { key: k, value: v });
        }
        h
    }
}
