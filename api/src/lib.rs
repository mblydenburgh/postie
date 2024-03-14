pub mod db;
pub mod domain;

use chrono::prelude::*;
use db::repository;
use domain::collection::Collection;
use domain::environment::EnvironmentFile;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Method,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{borrow::Borrow, error::Error, fs, str::FromStr};
use uuid::Uuid;

use crate::domain::{request::DBRequest, request_item::RequestHistoryItem, response::DBResponse};

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

#[derive(Debug)]
pub struct HttpRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<RequestBody>,
    pub environment: EnvironmentFile,
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

#[derive(Clone, Debug, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

#[derive(Debug)]
pub enum ResponseData {
    JSON(serde_json::Value),
    TEXT(String),
}

pub struct RequestCollection {
    pub name: String,
    pub requests: Vec<HttpRequest>,
}

pub struct Environment {
    pub name: String,
    pub variables: Vec<(String, String)>,
}

pub struct PostieApi {
    pub client: reqwest::Client,
    pub collection: Option<String>,
    pub environment: Option<String>,
    pub db: repository::PostieDb,
}

impl PostieApi {
    pub async fn new() -> Self {
        PostieApi {
            client: reqwest::Client::new(),
            collection: None,
            environment: None,
            db: repository::PostieDb::new().await,
        }
    }
    pub fn parse_collection(collection_json: &str) -> Collection {
        println!("Parsing collection from json");
        serde_json::from_str(&collection_json).expect("Failed to parse collection")
    }
    pub fn parse_environment(environment_json: &str) -> EnvironmentFile {
        println!("Parsing environment from json");
        serde_json::from_str(&environment_json).expect("Failed to parse environment")
    }
    pub fn read_file(path: &str) -> Result<String, Box<dyn Error>> {
        println!("Reading file: {}", path);
        Ok(fs::read_to_string(path)?)
    }
    pub async fn import_collection(path: &str) -> Result<String, String> {
        let mut api = PostieApi::new().await;
        let file_str = Self::read_file(path).unwrap();
        let collection = Self::parse_collection(&file_str);
        println!("Successfully parsed postman collection!");
        match api.db.save_collection(collection).await {
            Ok(_) => Ok(String::from("Import successful")),
            Err(_) => {
                println!("Error saving collection");
                Ok(String::from("Error saving collection"))
            }
        }
    }
    // TODO - better error handling
    pub async fn import_environment(path: &str) -> Result<String, Box<dyn Error + Send>> {
        let mut api = PostieApi::new().await;
        let file_str = Self::read_file(path).unwrap();
        let environment = Self::parse_environment(&file_str);
        println!("Successfully parsed postman environment!");
        match api.db.save_environment(environment).await {
            Ok(_) => Ok(String::from("Import successful")),
            Err(_) => {
                println!("Error saving enviornment");
                Ok(String::from("Error with importing"))
            }
        }
    }
    //TODO - connect to save button on ui to overwrite changes to existing env/collection
    pub fn save_environment(_input: Environment) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn save_collection(_input: RequestCollection) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub async fn load_environments() -> Result<Vec<EnvironmentFile>, Box<dyn Error + Send>> {
        let mut api = PostieApi::new().await;
        let envs = api.db.get_all_environments().await.unwrap();
        Ok(envs)
    }
    pub async fn load_collections() -> Result<Vec<Collection>, Box<dyn Error + Send>> {
        let mut api = PostieApi::new().await;
        let collections = api.db.get_all_collections().await.unwrap();
        Ok(collections)
    }
    pub async fn load_request_response_items(
    ) -> Result<Vec<RequestHistoryItem>, Box<dyn Error + Send>> {
        let mut api = PostieApi::new().await;
        let items = api.db.get_request_response_items().await.unwrap();
        Ok(items)
    }
    pub async fn load_saved_requests() -> Result<Vec<DBRequest>, Box<dyn Error + Send>> {
        let mut api = PostieApi::new().await;
        let requests = api.db.get_all_requests().await.unwrap();
        Ok(requests)
    }
    pub async fn load_saved_responses() -> Result<Vec<DBResponse>, Box<dyn Error + Send>> {
        let mut api = PostieApi::new().await;
        let responses = api.db.get_all_responses().await.unwrap();
        Ok(responses)
    }
    pub fn substitute_variables_in_url(environment: &EnvironmentFile, raw_url: String) -> String {
        println!("substituting env vars");
        if let Some(values) = environment.clone().values {
            let url = values.iter().fold(raw_url, |acc, env_value| {
                acc.replace(&format!("{{{{{}}}}}", env_value.key), &env_value.value)
            });
            println!("final url: {}", url);
            url
        } else {
            println!("env doesnt have values, returning original");
            raw_url
        }
    }
    pub async fn make_request(input: PostieRequest) -> Result<ResponseData, Box<dyn Error>> {
        let api = PostieApi::new().await;
        match input {
            PostieRequest::HTTP(input) => {
                println!("Submitting http request: {:?}", input);
                let method = match input.method {
                    HttpMethod::GET => Method::GET,
                    HttpMethod::POST => Method::POST,
                    HttpMethod::PUT => Method::PUT,
                    HttpMethod::PATCH => Method::PATCH,
                    HttpMethod::DELETE => Method::DELETE,
                    HttpMethod::HEAD => Method::HEAD,
                    HttpMethod::OPTIONS => Method::OPTIONS,
                };

                let mut headers = HeaderMap::new();
                if let Some(h) = input.headers.clone() {
                    for (key, value) in h {
                        let header_name = HeaderName::from_bytes(&key.as_bytes()).unwrap();
                        let header_value = HeaderValue::from_str(&value).unwrap();
                        headers.insert(header_name, header_value);
                    }
                };

                let url = Self::substitute_variables_in_url(&input.environment, input.url.clone());
                let mut req = api.client.request(method, url).headers(headers.clone());
                if let Some(ref request_body) = input.body {
                    req = match request_body.clone() {
                        RequestBody::JSON(j) => req.json(&j.clone()),
                        RequestBody::FORM(f) => req.form(&f.clone()),
                    };
                }

                let now: DateTime<Utc> = Utc::now();
                let sent_at = std::time::Instant::now();
                let res = req.send().await?;
                let response_time = sent_at.elapsed().as_millis();
                let res_headers = res.headers().clone();
                let res_status = res.status().clone();
                let res_type = &res_headers.get("content-type").unwrap().to_str().unwrap();
                let res_text = res.text().await?;
                if !res_type.starts_with("application/json")
                    && !res_type.starts_with("text/plain")
                    && !res_type.starts_with("text/html")
                {
                    // I couldn't figure out how to safely throw an error so I'm just returning this for now
                    println!(
                        "expected application/json text/html, or text/plain, got {}",
                        res_type
                    );
                    return Ok(ResponseData::JSON(
                        json!({"err": "unsupported response type!"}),
                    ));
                }

                let request_headers = input
                    .headers
                    .clone()
                    .unwrap()
                    .into_iter()
                    .map(|(key, value)| domain::request::RequestHeader { key, value })
                    .collect();
                let mut db = repository::PostieDb::new().await;
                let body = if let Some(req_body) = input.body {
                    match req_body {
                        RequestBody::JSON(j) => Some(j.to_string()),
                        RequestBody::FORM(f) => Some(f),
                    }
                } else {
                    None
                };
                let db_request = DBRequest {
                    id: input.id.to_string(),
                    body,
                    name: input.name.clone(),
                    method: input.method.to_string(),
                    url: input.url,
                    headers: request_headers,
                };
                let _ = db.save_request_history(&db_request).await?;
                let response_headers: Vec<domain::response::ResponseHeader> = res_headers
                    .borrow()
                    .into_iter()
                    .map(|(key, value)| domain::response::ResponseHeader {
                        key: String::from(HeaderName::as_str(&key)),
                        value: String::from(HeaderValue::to_str(&value).unwrap()),
                    })
                    .collect();
                let db_response = DBResponse {
                    id: Uuid::new_v4().to_string(),
                    status_code: res_status.as_u16(),
                    name: input.name.clone(),
                    headers: response_headers,
                    body: Some(res_text.clone()),
                };
                let _ = db.save_response(&db_response).await?;
                let _ = db
                    .save_request_response_item(&db_request, &db_response, &now, &response_time)
                    .await?;
                let response_data = if res_type.starts_with("application/json") {
                    let res_json = serde_json::from_str(&res_text)?;
                    ResponseData::JSON(res_json)
                } else {
                    ResponseData::TEXT(res_text)
                };
                Ok(response_data)
            }
            PostieRequest::OAUTH(input) => {
                println!("making ouath request");
                let auth_header_value =
                    base64::encode(format!("{}:{}", &input.client_id, &input.client_secret));
                let mut header_map = HeaderMap::new();
                let header_value = &format!("Basic {:?}", &auth_header_value);
                println!("auth header: {}", &header_value);
                header_map.insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(header_value).unwrap(),
                );
                header_map.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/x-www-form-urlencoded").unwrap(),
                );
                let mut req = api
                    .client
                    .request(Method::POST, input.access_token_url)
                    .headers(header_map);
                req = req.form(&input.request);
                let res = req.send().await?;
                println!("boom");
                println!("{:?}", res);
                Ok(ResponseData::JSON(res.json().await.unwrap()))
            }
        }
    }
}
