use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use api::domain::{
  collection::{Collection, CollectionFolder, CollectionRequest},
  environment::EnvironmentFile,
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
pub struct RemoveCollectionItemPayload {
  pub id: String,
  pub name: String,
}

#[derive(Debug)]
pub struct RemoveCollectionRequestPayload {
  pub col_id: String,
  pub folder_name: String,
  pub req_name: String,
}

#[derive(Debug)]
pub enum GuiEvent {
  SelectRequest {
    col_id: String,
    request: CollectionRequest,
  },
  SelectEnvironment(String),
  SubmitRequest(HttpRequest),
  SubmitOAuth2Request(OAuth2Request),
  RefreshCollections(Option<Vec<Collection>>),
  RefreshEnvironments(),
  RefreshRequestData(RefreshRequestDataPayload),
  SetActiveTab(String),
  SaveCollection(Collection),
  SaveEnvironment(),
  NewCollection(Option<String>),
  NewEnvironment(Option<String>),
  NewRequest(),
  AddRequestToCollection {
    col_id: String,
    folder: Option<CollectionFolder>,
    req: Option<HttpRequest>,
    selected_env: Option<EnvironmentFile>,
  },
  AddFolderToCollection {
    col_id: String,
    sub_folder: Option<CollectionFolder>,
    new_folder: CollectionFolder,
  },
  RemoveTab(Uuid),
  RemoveCollection(String),
  RemoveCollectionFolder(RemoveCollectionItemPayload),
  RemoveCollectionRequest(RemoveCollectionItemPayload),
  RemoveCollectionFolderRequest(RemoveCollectionRequestPayload),
}
