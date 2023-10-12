use std::error::Error;

use reqwest::{Client, IntoUrl};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

enum InputAction {
    SAVE_COLLECTION,
    SAVE_ENVIRONMENT,
    MAKE_REQUEST,
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

#[derive(Debug)]
pub struct HttpRequest {
    pub name: Option<String>,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
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
        Ok(serde_json::json!({
            "foo": "bar"
        }))
    }
}
pub fn save_environment(input: Environment) -> Result<(), Box<dyn Error>> {
    Ok(())
}
pub fn save_collection(input: RequestCollection) -> Result<(), Box<dyn Error>> {
    Ok(())
}
pub async fn submit_request(input: HttpRequest) -> Result<Value, Box<dyn Error>> {
    println!("making client");
    let client = Client::new();
    println!("Submitting request: {:?}", input);
    let res = match input.method {
        HttpMethod::GET => client.get(input.url).send().await?,
        HttpMethod::POST => todo!(),
        HttpMethod::PUT => todo!(),
        HttpMethod::PATCH => todo!(),
        HttpMethod::DELETE => todo!(),
        HttpMethod::OPTIONS => todo!(),
        HttpMethod::HEAD => todo!(),
    };

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
