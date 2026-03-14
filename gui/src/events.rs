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
  RefreshRequestData {
    request_history: Arc<RwLock<Vec<RequestHistoryItem>>>,
    responses: Arc<RwLock<HashMap<String, DBResponse>>>,
    requests: Arc<RwLock<HashMap<String, DBRequest>>>,
    tabs: Arc<RwLock<HashMap<Uuid, Tab>>>,
  },
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
  RemoveCollectionFolder {
    col_id: String,
    id: String,
  },
  RemoveCollectionRequest {
    col_id: String,
    id: String,
  },
  RemoveCollectionFolderRequest {
    col_id: String,
    folder_id: String,
    req_id: String,
  },
}
