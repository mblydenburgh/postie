use std::{borrow::BorrowMut, error::Error};

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::{Value, json};
use sqlx::{Connection, SqliteConnection};

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
        Ok(res_json)
    }
}

pub async fn initialize_db() -> Result<SqliteConnection, Box<dyn Error>> {
    println!("acquiring sqlite connection");
    let connection = SqliteConnection::connect("sqlite:postie.sqlite").await?;
    println!("{:?} sqlite connection established", connection);

    Ok(connection)
}
