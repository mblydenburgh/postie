use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::from_str;
use sqlx::{sqlite::SqliteRow, Connection, Row, SqliteConnection};
use uuid::Uuid;

use crate::domain::{
  collection::{Collection, CollectionAuth, CollectionInfo, CollectionItemOrFolder},
  environment::EnvironmentFile,
  request::{self, DBRequest, HttpMethod, RequestHeader, RequestHeaders},
  request_item::RequestHistoryItem,
  response::{DBResponse, ResponseHeader},
  tab::Tab,
};

pub async fn initialize_db() -> anyhow::Result<SqliteConnection> {
  println!("acquiring sqlite connection");
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 2 {
    println!("Usage: {} <sqlite db file>", args[0]);
    std::process::exit(1);
  }
  let db_path = &args[1];
  // if path does not exist, assume running locally and default to local copy
  if !Path::new(db_path).exists() {
    return Ok(SqliteConnection::connect("sqlite:postie.sqlite").await?);
  }
  let connection = SqliteConnection::connect(db_path).await?;
  println!("{:?} sqlite connection established", connection);

  Ok(connection)
}

pub struct PostieDb {
  pub connection: SqliteConnection,
}

impl PostieDb {
  pub async fn new() -> Self {
    PostieDb {
      connection: initialize_db()
        .await
        .ok()
        .expect("could not establish database connection"),
    }
  }

  pub async fn save_request_history(&mut self, request: &DBRequest) -> anyhow::Result<()> {
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
  ) -> anyhow::Result<()> {
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

  pub async fn get_request_response_items(&mut self) -> anyhow::Result<Vec<RequestHistoryItem>> {
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

  pub async fn save_environment(&mut self, environment: EnvironmentFile) -> anyhow::Result<()> {
    let mut transaction = self.connection.begin().await?;
    let value_json = match environment.values {
      None => serde_json::json!("[]"),
      Some(values) => serde_json::Value::String(serde_json::to_string(&values).unwrap()),
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

  pub async fn save_collection(&mut self, collection: Collection) -> anyhow::Result<()> {
    println!("Saving collection {:#?} to db", collection.info);
    let mut transaction = self.connection.begin().await?;
    let items_json = serde_json::to_string(&collection.item)?;
    let auth_json = serde_json::to_string(&collection.auth)?;
    _ = sqlx::query!(
      r#"
            INSERT OR REPLACE INTO collections (id, name, description, item, auth)
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

  pub async fn save_response(&mut self, response: &DBResponse) -> anyhow::Result<()> {
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

  pub async fn save_tab(&mut self, tab: &Tab) -> anyhow::Result<()> {
    println!("Saving tab to db: {:#?}", tab);
    let method = tab.method.to_string();
    let req_headers = serde_json::to_string(&tab.req_headers).unwrap();
    let res_headers = serde_json::to_string(&tab.res_headers).unwrap();
    let mut transaction = self.connection.begin().await?;
    _ = sqlx::query!(
            r#"
            INSERT INTO tabs (id, method, url, req_body, req_headers, res_status, res_body, res_headers)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET 
            method = $2, url = $3, req_body = $4, req_headers = $5, res_status = $6, res_body = $7, res_headers = $8
            "#,
            tab.id,
            method,
            tab.url,
            tab.req_body,
            req_headers,
            tab.res_status,
            tab.res_body,
            res_headers
        )
        .execute(&mut *transaction)
        .await
        .unwrap();
    transaction.commit().await?;
    Ok(())
  }

  pub async fn get_all_requests(&mut self) -> anyhow::Result<Vec<DBRequest>> {
    println!("getting all saved requests");
    let rows = sqlx::query("SELECT * FROM request")
      .map(|row: SqliteRow| {
        let id: String = row.get("id");
        let method: String = row.get("method");
        let url: String = row.get("url");
        let name: Option<String> = row.get("name");
        let raw_body: Option<String> = row.get("body");
        let raw_headers: String = row.get("headers");
        let mut body: Option<String> = None;
        let headers: Vec<request::RequestHeader> =
          serde_json::from_str::<Vec<RequestHeader>>(&raw_headers).unwrap();
        if let Some(body_str) = raw_body {
          body = Some(body_str)
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

  pub async fn get_all_collections(&mut self) -> anyhow::Result<Vec<Collection>> {
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

  pub async fn get_all_responses(&mut self) -> anyhow::Result<Vec<DBResponse>> {
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
          None
        } else {
          Some(String::from(&raw_body))
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

  pub async fn get_all_environments(&mut self) -> anyhow::Result<Vec<EnvironmentFile>> {
    println!("getting all envs");
    let rows = sqlx::query("SELECT * FROM environment")
      .map(|row: SqliteRow| {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let raw_values: Option<String> = row.get("values");
        if let Some(values_json) = raw_values {
          let values_str: Result<String, serde_json::Error> = serde_json::from_str(&values_json);
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

  pub async fn get_all_tabs(&mut self) -> anyhow::Result<Vec<Tab>> {
    println!("getting all tabs");
    let rows = sqlx::query("SELECT * FROM tabs")
      .map(|row: SqliteRow| {
        let id: String = row.get("id");
        let url: String = row.get("url");
        let raw_req_body: Option<String> = row.get("req_body");
        let raw_res_body: Option<String> = row.get("res_body");
        let method: String = row.get("method");
        let res_status: Option<String> = row.get("res_status");
        let raw_req_headers: String = row.get("req_headers");
        println!("raw_req_headers: {:?}", raw_req_headers);
        let mut req_body: Option<String> = None;
        let mut res_body: String = "".into();
        let headers: RequestHeaders = serde_json::from_str::<RequestHeaders>(&raw_req_headers)
          .unwrap_or(RequestHeaders(vec![]));
        if let Some(body_str) = raw_req_body {
          req_body = Some(body_str)
        }
        if let Some(body) = raw_res_body {
          res_body = body
        }
        Tab {
          id,
          url,
          req_body: req_body.unwrap_or_default(),
          method: match method.as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            _ => HttpMethod::GET,
          },
          res_status,
          req_headers: headers,
          res_body,
          res_headers: RequestHeaders(vec![]),
        }
      })
      .fetch_all(&mut self.connection)
      .await
      .unwrap();
    Ok(rows)
  }

  pub async fn delete_tab(&mut self, tab_id: Uuid) -> anyhow::Result<()> {
    let id = tab_id.to_string();
    sqlx::query!("DELETE FROM tabs WHERE id = $1", id)
      .execute(&mut self.connection)
      .await?;
    Ok(())
  }

  pub async fn delete_collection(&mut self, collection_id: String) -> anyhow::Result<()> {
    let id = collection_id.to_string();
    sqlx::query!("DELETE FROM collections WHERE id = $1", id)
      .execute(&mut self.connection)
      .await?;
    Ok(())
  }
}
