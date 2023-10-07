use std::error::Error;

use serde_json::Value;

enum InputAction {
    SAVE_COLLECTION,
    SAVE_ENVIRONMENT,
    MAKE_REQUEST,
}

enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

pub struct HttpRequest {
    name: String,
    method: HttpMethod,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

pub struct RequestCollection {
    name: String,
    requests: Vec<HttpRequest>,
}

pub struct Environment {
    name: String,
    variables: Vec<(String, String)>,
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
