pub mod domain;

use std::{borrow::BorrowMut, error::Error, fs, sync::Arc};

use domain::collection::Collection;
use domain::environment::EnvironmentFile;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use uuid::Uuid;

use crate::domain::request::DBRequest;

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
    pub async fn import_collection(path: &str) -> Result<(), Box<dyn Error + Send>> {
        let file_str = Self::read_file(path).unwrap();
        let _collection = Self::parse_collection(&file_str);
        println!("Successfully parsed postman collection!");
        Ok(())
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
    pub async fn make_request(input: HttpRequest) -> Result<Value, Box<dyn Error>> {
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

        let res = req.send().await?;
        let res_type = res.headers().get("content-type").unwrap().to_str().unwrap();
        if !res_type.starts_with("application/json") {
            // I couldn't figure out how to safely throw an error so I'm just returning this for now
            println!("expected application/json, got {}", res_type);
            return Ok(json!({"msg": "not JSON!"}));
        }

        let res_str = res.text().await?;
        let res_json = serde_json::from_str(&res_str).unwrap_or_default();

        let request_headers = input
            .headers
            .clone()
            .unwrap()
            .into_iter()
            .map(|(key, value)| domain::request::RequestHeader { key, value })
            .collect();
        let mut db = PostieDb::new().await;
        let _ = db
            .save_request_history(DBRequest {
                id: input.id.to_string(),
                body: input.body,
                name: input.name,
                method: input.method.to_string(),
                url: input.url,
                headers: request_headers,
            })
            .await?;

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
    pub connection: SqliteConnection,
}

impl PostieDb {
    pub async fn new() -> Self {
        PostieDb {
            connection: initialize_db().await.ok().unwrap(),
        }
    }

    pub async fn save_request_history(&mut self, request: DBRequest) -> Result<(), Box<dyn Error>> {
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

    pub async fn get_request_history(
        &mut self
    ) -> Result<Arc<[DBRequest]>, Box<dyn Error>> {
        let rows = sqlx::query!("SELECT * FROM request_history")
            .map(|row: SqliteRow| {
                let id: String = row.get("id");
                let request_id: String = row.get("request_id");
                let response_id: String = row.get("response_id");
                let sent_at: String = row.get("sent_at");
                let response_time_ms: String = row.get("response_time_ms");
                DBRequest {
                    id,
                    request_id,
                    response_id,
                    response_time_ms,
                    sent_at
                }
            })
            .fetch_all(&mut self.connection)
            .await
            .unwrap();
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
