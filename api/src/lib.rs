pub mod domain;

use std::{error::Error, fs};

use domain::collection::Collection;
use domain::environment::EnvironmentFile;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Connection, SqliteConnection};
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
    pub environment: EnvironmentFile,
}

pub struct RequestCollection {
    pub name: String,
    pub requests: Vec<HttpRequest>,
}

pub struct Environment {
    pub name: String,
    pub variables: Vec<(String, String)>,
}

#[derive(Clone)]
pub struct PostieApi {
    pub client: reqwest::Client,
    pub collection: Option<String>,
    pub environment: Option<String>,
}

impl PostieApi {
    pub fn new() -> Self {
        PostieApi {
            client: reqwest::Client::new(),
            collection: None,
            environment: None,
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
    pub fn save_environment(_input: Environment) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn save_collection(_input: RequestCollection) -> Result<(), Box<dyn Error>> {
        Ok(())
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
    pub async fn make_request(&self, input: HttpRequest) -> Result<Value, Box<dyn Error>> {
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

        let url = Self::substitute_variables_in_url(&input.environment, input.url.clone());
        let mut req = self.client.request(method, url).headers(headers);
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
