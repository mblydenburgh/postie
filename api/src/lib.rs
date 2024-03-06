pub mod domain;

use chrono::prelude::*;
use domain::collection::Collection;
use domain::environment::EnvironmentFile;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use snailquote::unescape;
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use std::{borrow::Borrow, error::Error, fs, str::FromStr};
use uuid::Uuid;

use crate::domain::{
    collection::{CollectionAuth, CollectionInfo, CollectionItemOrFolder},
    request::{self, DBRequest, RequestHeader},
    request_item::RequestHistoryItem,
    response::{DBResponse, ResponseHeader},
};

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
pub struct HttpRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<Value>,
    pub environment: EnvironmentFile,
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
    pub db: PostieDb,
}

impl PostieApi {
    pub async fn new() -> Self {
        PostieApi {
            client: reqwest::Client::new(),
            collection: None,
            environment: None,
            db: PostieDb::new().await,
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
    pub async fn import_collection(path: &str) -> Result<String, Box<dyn Error + Send>> {
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
    pub async fn make_request(input: HttpRequest) -> Result<ResponseData, Box<dyn Error>> {
        let api = PostieApi::new().await;
        println!("Submitting request: {:?}", input);
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
        if input.body.is_some() {
            req = req.json(&input.body.clone().unwrap_or_default());
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
        let mut db = PostieDb::new().await;
        let db_request = DBRequest {
            id: input.id.to_string(),
            body: input.body,
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
}

pub async fn initialize_db() -> Result<SqliteConnection, Box<dyn Error>> {
    println!("acquiring sqlite connection");
    let connection = SqliteConnection::connect("sqlite:postie.sqlite").await?;
    println!("{:?} sqlite connection established", connection);

    Ok(connection)
}

pub struct PostieDb {
    pub connection: SqliteConnection,
}

impl PostieDb {
    pub async fn new() -> Self {
        PostieDb {
            connection: initialize_db().await.ok().unwrap(),
        }
    }

    pub async fn save_request_history(
        &mut self,
        request: &DBRequest,
    ) -> Result<(), Box<dyn Error>> {
        println!("got request: {:?}", request);
        let mut transaction = self.connection.begin().await?;
        let header_json = serde_json::to_string(&request.headers)?;
        let _request = sqlx::query!(
            r#"
            INSERT INTO request (id, method, url, name, headers, body) 
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            request.id,
            request.method,
            request.url,
            request.name,
            header_json,
            request.body
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        println!("transaction committed");

        Ok(())
    }

    pub async fn save_request_response_item(
        &mut self,
        request: &DBRequest,
        response: &DBResponse,
        sent_at: &DateTime<Utc>,
        response_time: &u128,
    ) -> Result<(), Box<dyn Error>> {
        println!("Saving request response history item");
        let mut transaction = self.connection.begin().await?;
        let id = Uuid::new_v4().to_string();
        let converted_sent = sent_at.to_string();
        let converted_response_time = response_time.to_string();
        _ = sqlx::query!(
            r#"
            INSERT INTO request_history (id, request_id, response_id, sent_at, response_time_ms)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            request.id,
            response.id,
            converted_sent,
            converted_response_time
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_request_response_items(
        &mut self,
    ) -> Result<Vec<RequestHistoryItem>, Box<dyn Error>> {
        println!("getting all request response items");
        let rows = sqlx::query("SELECT * FROM request_history")
            .map(|row: SqliteRow| {
                let id: String = row.get("id");
                let request_id: String = row.get("request_id");
                let response_id: String = row.get("response_id");
                let sent_at: String = row.get("sent_at");
                let response_time: String = row.get("response_time_ms");
                RequestHistoryItem {
                    id,
                    request_id,
                    response_id,
                    response_time: from_str::<usize>(&response_time).unwrap(),
                    sent_at,
                }
            })
            .fetch_all(&mut self.connection)
            .await
            .unwrap();
        Ok(rows)
    }

    pub async fn save_environment(
        &mut self,
        environment: EnvironmentFile,
    ) -> Result<(), Box<dyn Error>> {
        let mut transaction = self.connection.begin().await?;
        let value_json = match environment.values {
            None => json!("[]"),
            Some(values) => Value::String(serde_json::to_string(&values).unwrap()),
        };
        let uuid = Uuid::new_v4().to_string();
        _ = sqlx::query!(
            r#"
            INSERT INTO environment (id, name, `values`)
            VALUES ($1, $2, $3)
            "#,
            uuid,
            environment.name,
            value_json
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_collection(&mut self, collection: Collection) -> Result<(), Box<dyn Error>> {
        println!("Saving collection to db");
        let mut transaction = self.connection.begin().await?;
        let items_json = serde_json::to_string(&collection.item)?;
        let auth_json = serde_json::to_string(&collection.auth)?;
        _ = sqlx::query!(
            r#"
            INSERT INTO collections (id, name, description, item, auth)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            collection.info.id,
            collection.info.name,
            collection.info.description,
            items_json,
            auth_json
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_response(&mut self, response: &DBResponse) -> Result<(), Box<dyn Error>> {
        println!("Saving response to db");
        let mut transaction = self.connection.begin().await?;
        let header_json = serde_json::to_string(&response.headers)?;
        _ = sqlx::query!(
            r#"
            INSERT INTO response (id, name, status_code, headers, body)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            response.id,
            response.name,
            response.status_code,
            header_json,
            response.body
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_all_requests(&mut self) -> Result<Vec<DBRequest>, Box<dyn Error>> {
        println!("getting all saved requests");
        let rows = sqlx::query("SELECT * FROM request")
            .map(|row: SqliteRow| {
                let id: String = row.get("id");
                let method: String = row.get("method");
                let url: String = row.get("url");
                let name: Option<String> = row.get("name");
                let raw_body: Option<String> = row.get("body");
                let raw_headers: String = row.get("headers");
                let mut body: Option<serde_json::Value> = None;
                let headers: Vec<request::RequestHeader> =
                    serde_json::from_str::<Vec<RequestHeader>>(&raw_headers).unwrap();
                if let Some(body_str) = raw_body {
                    body = serde_json::from_str(&body_str).unwrap()
                }
                DBRequest {
                    id,
                    method,
                    url,
                    name,
                    headers,
                    body,
                }
            })
            .fetch_all(&mut self.connection)
            .await
            .unwrap();
        Ok(rows)
    }

    pub async fn get_all_collections(&mut self) -> Result<Vec<Collection>, Box<dyn Error>> {
        println!("getting all saved collections");
        let rows = sqlx::query("SELECT * from collections")
            .map(|row: SqliteRow| {
                let id: String = row.get("id");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let raw_item: String = row.get("item");
                let raw_auth: Option<String> = row.get("auth");
                let item: Vec<CollectionItemOrFolder> = serde_json::from_str(&raw_item).unwrap();
                let auth: Option<CollectionAuth> = match raw_auth {
                    Some(a) => serde_json::from_str(&a).unwrap(),
                    None => None,
                };
                Collection {
                    info: CollectionInfo {
                        id,
                        name,
                        description,
                    },
                    item,
                    auth,
                }
            })
            .fetch_all(&mut self.connection)
            .await
            .unwrap();
        Ok(rows)
    }

    pub async fn get_all_responses(&mut self) -> Result<Vec<DBResponse>, Box<dyn Error>> {
        println!("getting all saved responses");
        let rows = sqlx::query("SELECT * from response")
            .map(|row: SqliteRow| {
                let id: String = row.get("id");
                let status_code: u16 = row.get("status_code");
                let name: Option<String> = row.get("name");
                let raw_headers: String = row.get("headers");
                let raw_body: String = row.get("body");
                let headers = serde_json::from_str::<Vec<ResponseHeader>>(&raw_headers).unwrap();
                let body = if raw_body.is_empty() {
                    None // or handle the empty case as desired
                } else {
                    let escaped = unescape(&raw_body).unwrap();
                    Some(escaped)
                };
                DBResponse {
                    id,
                    status_code,
                    name,
                    headers,
                    body,
                }
            })
            .fetch_all(&mut self.connection)
            .await
            .unwrap();
        Ok(rows)
    }

    pub async fn get_all_environments(&mut self) -> Result<Vec<EnvironmentFile>, Box<dyn Error>> {
        println!("getting all envs");
        let rows = sqlx::query("SELECT * FROM environment")
            .map(|row: SqliteRow| {
                let id: String = row.get("id");
                let name: String = row.get("name");
                let raw_values: Option<String> = row.get("values");
                if let Some(values_json) = raw_values {
                    let values_str: Result<String, serde_json::Error> =
                        serde_json::from_str(&values_json);
                    match values_str {
                        Ok(str) => {
                            let values = serde_json::from_str(&str).expect("couldnt parse string");
                            EnvironmentFile {
                                id,
                                name,
                                values: Some(values),
                            }
                        }
                        Err(e) => {
                            println!("error: {:#?}", e);
                            EnvironmentFile {
                                id,
                                name,
                                values: None,
                            }
                        }
                    }
                } else {
                    EnvironmentFile {
                        id,
                        name,
                        values: None,
                    }
                }
            })
            .fetch_all(&mut self.connection)
            .await
            .unwrap();

        for row in rows.clone().into_iter() {
            println!("row: {:?}", &row);
        }
        Ok(rows)
    }
}
