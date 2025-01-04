use std::str::FromStr;

use crate::domain::{environment, request};

use serde::{Deserialize, Serialize};
use sqlx;
use uuid::Uuid;

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

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD,
}
impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct HttpMethodParseError;
impl FromStr for HttpMethod {
    type Err = HttpMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "DELETE" => Ok(HttpMethod::DELETE),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "HEAD" => Ok(HttpMethod::HEAD),
            _ => Err(HttpMethodParseError),
        }
    }
}

#[derive(Debug)]
pub enum PostieRequest {
    HTTP(HttpRequest),
    OAUTH(OAuth2Request),
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub tab_id: Uuid,
    pub id: Uuid,
    pub name: Option<String>,
    pub method: request::HttpMethod,
    pub url: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<RequestBody>,
    pub environment: environment::EnvironmentFile,
}

#[derive(Clone, Debug)]
pub enum RequestBody {
    JSON(serde_json::Value),
    FORM(String),
}

#[derive(Debug)]
pub struct OAuth2Request {
    pub access_token_url: String,
    pub refresh_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub request: OAuthRequestBody,
}

#[derive(Debug, Serialize)]
pub struct OAuthRequestBody {
    pub grant_type: String,
    pub scope: String,
    pub audience: String,
}
