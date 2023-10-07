use std::error::Error;

use serde::{Serialize, Deserialize};
use serde_json::Value;

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
    HEAD
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
        print!("Submitting request: {:?}", input);
        Ok(serde_json::json!({
            "foo": "bar"
        }))
    }
