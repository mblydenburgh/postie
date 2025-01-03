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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RequestHeadersIterator<'a> {
    headers: &'a [RequestHeader],
    index: usize,
}

impl<'a> Iterator for RequestHeadersIterator<'a> {
    type Item = &'a RequestHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.headers.len() {
            let result = &self.headers[self.index];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a RequestHeaders {
    type Item = &'a RequestHeader;
    type IntoIter = RequestHeadersIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RequestHeadersIterator {
            headers: &self.0,
            index: 0,
        }
    }
}
