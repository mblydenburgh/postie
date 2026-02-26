use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use api::domain::{
  collection::Collection,
  request::{DBRequest, HttpRequest, OAuth2Request},
  request_item::RequestHistoryItem,
  response::DBResponse,
  tab::Tab,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct RefreshRequestDataPayload {
  pub request_history: Arc<RwLock<Vec<RequestHistoryItem>>>,
  pub responses: Arc<RwLock<HashMap<String, DBResponse>>>,
  pub requests: Arc<RwLock<HashMap<String, DBRequest>>>,
  pub tabs: Arc<RwLock<HashMap<Uuid, Tab>>>,
}

#[derive(Debug)]
pub enum GuiEvent {
  SubmitRequest(HttpRequest),
  SubmitOAuth2Request(OAuth2Request),
  RefreshCollections(),
  RefreshEnvironments(),
  RefreshRequestData(RefreshRequestDataPayload),
  SetActiveTab(String),
  SaveCollection(Collection),
  SaveEnvironment(),
  NewCollection(Option<String>),
  NewEnvironment(Option<String>),
  NewRequest(),
  RemoveTab(Uuid),
}
