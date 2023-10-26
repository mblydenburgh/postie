pub mod domain;

use std::{error::Error, fs};

use domain::collection::Collection;
use domain::environment::EnvironmentFile;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Connection, SqliteConnection, Executor};
use uuid::Uuid;

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

#[derive(Debug)]
pub struct HttpRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<Value>,
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
    pub environment: Option<String>,
    pub collection: Option<String>,
}

impl PostieApi {
    pub fn new() -> Self {
        PostieApi {
            environment: None,
            collection: None,
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
    pub async fn import_collection(path: &str) -> Result<(), Box<dyn Error + Send>> {
        let file_str = Self::read_file(path).unwrap();
        let _collection = Self::parse_collection(&file_str);
        println!("Successfully parsed postman collection!");
        Ok(())
    }
    pub async fn import_environment(path: &str) -> Result<(), Box<dyn Error + Send>> {
        let file_str = Self::read_file(path).unwrap();
        let _collection = Self::parse_environment(&file_str);
        println!("Successfully parsed postman environment!");
        Ok(())
    }
    pub fn save_environment(input: Environment) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn save_collection(input: RequestCollection) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub async fn make_request(input: HttpRequest) -> Result<Value, Box<dyn Error>> {
        let client = Client::new();
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
        if let Some(h) = input.headers {
            for (key, value) in h {
                let header_name = HeaderName::from_bytes(&key.as_bytes()).unwrap();
                let header_value = HeaderValue::from_str(&value).unwrap();
                headers.insert(header_name, header_value);
            }
        };
        let mut req = client.request(method, input.url).headers(headers);
        if input.body.is_some() {
            req = req.json(&input.body.unwrap_or_default());
        }

        let res = req.send().await?;
        let res_type = res.headers().get("content-type").unwrap().to_str().unwrap();
        if !res_type.starts_with("application/json") {
            // I couldn't figure out how to safely throw an error so I'm just returning this for now
            println!("expected application/json, got {}", res_type);
            return Ok(json!({"msg": "not JSON!"}));
        }

        let res_str = res.text().await?;
        let res_json = serde_json::from_str(&res_str).unwrap_or_default();

        let mut db = PostieDb::new().await;
        let _ = db.save_request_history(DBRequest { id: input.id.to_string(), body: "hi".to_string(), name: "my request".to_string(), method: "POST".to_string(), url: "https://google.com".to_string(), headers: json!({ "x-api-key": "foo" }).to_string() }).await?;

        Ok(res_json)
    }
}

pub async fn initialize_db() -> Result<SqliteConnection, Box<dyn Error>> {
    println!("acquiring sqlite connection");
    let connection = SqliteConnection::connect("sqlite:postie.sqlite").await?;
    println!("{:?} sqlite connection established", connection);

    Ok(connection)
}

pub struct PostieDb {
    pub connection: SqliteConnection
}

#[derive(Debug)]
pub struct DBRequest {
    id: String,
    method: String,
    url: String,
    name: String,
    headers: String,
    body: String
}

struct DBResponse {
    id: String,
    status_code: u8,
    name: String,
    headers: String,
    body: String
}

struct DBRequestHistoryItem {
    id: String,
    request_id: String,
    response_id: String,
    sent_at: String,
    response_time: u32
}

impl PostieDb {
    pub async fn new() -> Self {
        PostieDb { connection: initialize_db().await.ok().unwrap() }
    }

    pub async fn save_request_history(&mut self, request: DBRequest) -> Result<(), Box<dyn Error>> {
        println!("got request: {:?}", request);
        let mut transaction = self.connection.begin().await?;
        // let res = sqlx::query!("SELECT * FROM request_history").fetch_all(&mut *transaction).await;

        let _request = sqlx::query_as!(DBRequest, "INSERT INTO request (id, method, url, name, headers, body) VALUES ($1, $2, $3, $4, $5, $6)", request.id, request.method, request.url, request.name, request.headers, request.body).execute(&mut *transaction).await?;

        transaction.commit().await?;
        println!("transaction committed");

        Ok(())
    }
}
