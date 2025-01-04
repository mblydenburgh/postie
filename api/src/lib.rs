pub mod db;
pub mod domain;
pub mod utilities;

use chrono::prelude::*;
use db::repository;
use domain::environment::EnvironmentFile;
use domain::request::RequestHeaders;
use domain::{
    collection::{
        Collection, CollectionItem, CollectionItemOrFolder, CollectionRequest,
        CollectionRequestHeader, CollectionUrl,
    },
    request::{HttpRequest, PostieRequest, RequestBody},
    tab::Tab,
};
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Method,
};
use serde::Deserialize;
use serde_json::json;
use std::{borrow::Borrow, fs};
use uuid::Uuid;

use crate::domain::{request::DBRequest, request_item::RequestHistoryItem, response::DBResponse};

#[derive(Clone, Debug, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status: String,
    pub data: ResponseData,
}

#[derive(Clone, Debug)]
pub enum ResponseData {
    JSON(serde_json::Value),
    TEXT(String),
    XML(String),
    UNKNOWN(String),
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
        serde_json::from_str(collection_json).expect("Failed to parse collection")
    }
    pub fn parse_environment(environment_json: &str) -> EnvironmentFile {
        println!("Parsing environment from json");
        serde_json::from_str(environment_json).expect("Failed to parse environment")
    }
    pub fn read_file(path: &str) -> anyhow::Result<String> {
        println!("Reading file: {}", path);
        Ok(fs::read_to_string(path)?)
    }
    pub async fn import_collection(path: &str) -> anyhow::Result<String> {
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
    pub async fn add_request_to_collection(
        id: &str,
        req: HttpRequest,
        folder_name: String,
    ) -> anyhow::Result<()> {
        let mut api = PostieApi::new().await;
        println!("finding collection {id} to update");
        let collections = api.db.get_all_collections().await?;
        for mut collection in collections {
            if collection.info.id == id {
                println!("adding request to {folder_name}");
                for item in &mut collection.item {
                    if let CollectionItemOrFolder::Folder(ref mut folder) = item {
                        if folder.name == folder_name {
                            println!("found matching folder name, updating collection");
                            let mut res: Vec<CollectionRequestHeader> = vec![];
                            let headers: Vec<CollectionRequestHeader> = req
                                .headers
                                .clone()
                                .map(|headers| {
                                    for h in headers {
                                        res.push(CollectionRequestHeader {
                                            key: h.0,
                                            value: h.1,
                                            r#type: String::from(""),
                                        });
                                    }
                                    res
                                })
                                .unwrap();
                            folder
                                .item
                                .push(CollectionItemOrFolder::Item(CollectionItem {
                                    name: req.clone().url,
                                    request: CollectionRequest {
                                        auth: None,
                                        body: Some(domain::collection::RequestBody {
                                            mode: String::from(""),
                                            raw: None,
                                            options: None,
                                        }),
                                        header: Some(headers),
                                        method: req.method.to_string(),
                                        url: CollectionUrl {
                                            raw: req.clone().url,
                                            path: None,
                                            host: None,
                                        },
                                    },
                                }));
                        }
                    }
                }
                let updated_items = collection.item;
                let updated = Collection {
                    info: collection.info,
                    item: updated_items,
                    auth: collection.auth,
                };
                api.db.save_collection(updated).await?;
            }
        }
        Ok(())
    }
    // TODO - better error handling
    pub async fn import_environment(path: &str) -> anyhow::Result<String> {
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
    pub async fn save_environment(input: EnvironmentFile) -> anyhow::Result<()> {
        let mut api = PostieApi::new().await;
        match api.db.save_environment(input).await {
            Ok(_) => Ok(()),
            Err(_) => {
                println!("Error saving environment");
                Ok(())
            }
        }
    }
    pub async fn save_collection(input: Collection) -> anyhow::Result<()> {
        let mut api = PostieApi::new().await;
        match api.db.save_collection(input).await {
            Ok(_) => Ok(()),
            Err(_) => {
                println!("Error saving collection");
                Ok(())
            }
        }
    }
    pub async fn load_environments() -> anyhow::Result<Vec<EnvironmentFile>> {
        let mut api = PostieApi::new().await;
        let envs = api.db.get_all_environments().await.unwrap();
        Ok(envs)
    }
    pub async fn load_collections() -> anyhow::Result<Vec<Collection>> {
        let mut api = PostieApi::new().await;
        let collections = api.db.get_all_collections().await.unwrap();
        Ok(collections)
    }
    pub async fn load_tabs() -> anyhow::Result<Vec<Tab>> {
        let mut api = PostieApi::new().await;
        let tabs = api.db.get_all_tabs().await.unwrap();
        Ok(tabs)
    }
    pub async fn load_request_response_items() -> anyhow::Result<Vec<RequestHistoryItem>> {
        let mut api = PostieApi::new().await;
        let items = api.db.get_request_response_items().await.unwrap();
        Ok(items)
    }
    pub async fn load_saved_requests() -> anyhow::Result<Vec<DBRequest>> {
        let mut api = PostieApi::new().await;
        let requests = api.db.get_all_requests().await.unwrap();
        Ok(requests)
    }
    pub async fn load_saved_responses() -> anyhow::Result<Vec<DBResponse>> {
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
    pub async fn make_request(input: PostieRequest) -> anyhow::Result<Response> {
        let api = PostieApi::new().await;
        match input {
            // request and save http request
            PostieRequest::HTTP(input) => {
                println!("Submitting http request: {:?}", input);
                let method = utilities::request::convert_http_method(input.method.clone());

                let mut headers = HeaderMap::new();
                if let Some(h) = input.headers.clone() {
                    for (key, value) in h {
                        let header_name = HeaderName::from_bytes(key.as_bytes()).unwrap();
                        let header_value = HeaderValue::from_str(&value).unwrap();
                        headers.insert(header_name, header_value);
                    }
                };

                let url = Self::substitute_variables_in_url(
                    &input.environment.clone(),
                    input.url.clone(),
                );
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
                let res_status = res.status();
                let res_type = res_headers.get("content-type").unwrap().to_str().unwrap();
                let res_text = res.text().await?;
                if !res_type.starts_with("application/json")
                    && !res_type.starts_with("text/plain")
                    && !res_type.starts_with("text/html")
                    && !res_type.starts_with("application/xml")
                    && !res_type.starts_with("text/xml")
                {
                    // I couldn't figure out how to safely throw an error so I'm just returning this for now
                    println!(
                        "expected application/json, application/xml, text/xml, text/html, or text/plain, got {}",
                        res_type
                    );
                    return Ok(Response {
                        data: ResponseData::JSON(
                            json!({"err": println!("unsupported response type {}!", res_type)}),
                        ),
                        status: res_status.to_string(),
                    });
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
                    url: input.url.clone(),
                    headers: request_headers,
                };
                db.save_request_history(&db_request).await?;
                let response_headers: Vec<domain::response::ResponseHeader> = res_headers
                    .borrow()
                    .into_iter()
                    .map(|(key, value)| domain::response::ResponseHeader {
                        key: String::from(HeaderName::as_str(key)),
                        value: String::from(HeaderValue::to_str(value).unwrap()),
                    })
                    .collect();
                let db_response = DBResponse {
                    id: Uuid::new_v4().to_string(),
                    status_code: res_status.as_u16(),
                    name: input.name.clone(),
                    headers: response_headers,
                    body: Some(res_text.clone()),
                };
                db.save_response(&db_response).await?;
                db
                    .save_request_response_item(&db_request, &db_response, &now, &response_time)
                    .await?;
                let res_data = match res_type {
                    "application/json" => {
                        let res_json = serde_json::from_str(&res_text)?;
                        ResponseData::JSON(res_json)
                    }
                    "application/xml" => {
                        // TODO - validate xml, currently trying to use serder_xml_rs::from_str()
                        // fails
                        ResponseData::XML(res_text)
                    }
                    "text/plain" => ResponseData::TEXT(res_text),
                    "text/html" => ResponseData::TEXT(res_text),
                    "text/xml" => {
                        // TODO - validate xml, currently trying to use serder_xml_rs::from_str()
                        // fails
                        ResponseData::XML(res_text)
                    }
                    _ => ResponseData::UNKNOWN("".into()),
                };
                let res_body = match &res_data {
                    ResponseData::JSON(j) => j.to_string(),
                    ResponseData::TEXT(t) => t.to_string(),
                    ResponseData::XML(x) => x.to_string(),
                    ResponseData::UNKNOWN(t) => t.to_string(),
                };
                let updated_tab = Tab {
                    id: input.tab_id.to_string(),
                    method: input.method.clone(),
                    url: input.url.clone(),
                    req_body: "".into(),
                    req_headers: RequestHeaders(vec![]),
                    res_status: Some(res_status.to_string()),
                    res_body,
                    res_headers: RequestHeaders(vec![]),
                };
                db.save_tab(&updated_tab).await?;
                Ok(Response {
                    data: res_data,
                    status: res_status.to_string(),
                })
            }
            // if making an oauth token request, dont save to db
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
                println!("{:?}", res);
                let status = res.status().to_string();
                Ok(Response {
                    data: ResponseData::JSON(res.json().await.unwrap()),
                    status,
                })
            }
        }
    }
    pub async fn delete_tab(tab_id: Uuid) -> anyhow::Result<()> {
        let mut db = repository::PostieDb::new().await;
        db.delete_tab(tab_id).await?;
        Ok(())
    }
}
