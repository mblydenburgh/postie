pub mod components;
mod events;

use anyhow::Error;
use api::{
  domain::{
    collection::Collection,
    environment::{EnvironmentFile, EnvironmentValue},
    request::{DBRequest, OAuth2Request, OAuthRequestBody, PostieRequest},
    request_item::RequestHistoryItem,
    response::{DBResponse, ResponseData},
    tab::Tab,
  },
  PostieApi,
};
use components::{
  content_header_panel::ContentHeaderPanel, content_panel::ContentPanel,
  content_side_panel::content_side_panel, import_modal::import_modal, menu_panel::MenuPanel,
  new_modal::new_modal, save_window::save_window, side_panel::side_panel,
};
use eframe::{egui, App, NativeOptions};
use std::{
  cell::RefCell,
  collections::HashMap,
  rc::Rc,
  sync::{Arc, Mutex},
};
use tokio::sync::RwLock;
use uuid::Uuid;

// Holds app state that needs to be thread safe
pub struct ThreadSafeState {
  pub api: Arc<RwLock<PostieApi>>,
  pub environments: Arc<RwLock<Vec<EnvironmentFile>>>,
  pub collections: Arc<RwLock<Vec<Collection>>>,
  pub tabs: Arc<RwLock<HashMap<String, Tab>>>,
  pub active_tab: Arc<RwLock<Tab>>,
  pub saved_requests: Arc<RwLock<HashMap<String, DBRequest>>>,
  pub saved_responses: Arc<RwLock<HashMap<String, DBResponse>>>,
  pub request_history_items: Arc<RwLock<Vec<RequestHistoryItem>>>,
  pub response: Arc<RwLock<Option<ResponseData>>>,
  pub oauth_response: Arc<RwLock<Option<ResponseData>>>,
  pub res_status: Arc<RwLock<String>>,
  pub received_token: Arc<Mutex<bool>>,
  pub is_requesting: Arc<RwLock<Option<bool>>>,
  pub import_result: Arc<Mutex<Option<String>>>,
}

pub struct GuiState {
  pub headers: Rc<RefCell<Vec<(bool, String, String)>>>,
  pub selected_history_item: Rc<RefCell<Option<api::domain::request_item::RequestHistoryItem>>>,
  pub selected_environment: Rc<RefCell<api::domain::environment::EnvironmentFile>>,
  pub selected_collection: Rc<RefCell<Option<api::domain::collection::Collection>>>,
  pub selected_http_method: api::domain::request::HttpMethod,
  pub selected_auth_mode: api::domain::ui::AuthMode,
  pub selected_save_window_collection: Option<api::domain::collection::Collection>,
  pub selected_save_window_folder: Option<String>,
  pub api_key: String,
  pub api_key_name: String,
  pub bearer_token: String,
  pub oauth_config: OAuth2Request,
  pub oauth_token: String,
  pub selected_request: Rc<RefCell<Option<api::domain::collection::CollectionRequest>>>,
  pub url: String,
  pub body_str: String,
  pub import_window_open: RwLock<bool>,
  pub new_window_open: RwLock<bool>,
  pub new_window_mode: RwLock<api::domain::ui::NewWindowMode>,
  pub new_name: String,
  pub save_window_open: Rc<RwLock<bool>>,
  pub import_mode: RwLock<api::domain::ui::ImportMode>,
  pub import_file_path: String,
  pub env_vars: Rc<RefCell<Vec<EnvironmentValue>>>,
  pub active_window: RwLock<api::domain::ui::ActiveWindow>,
  pub request_window_mode: RwLock<api::domain::ui::RequestWindowMode>,
}

pub struct Gui {
  pub worker_state: ThreadSafeState,
  pub gui_state: GuiState,
  pub event_tx: tokio::sync::mpsc::Sender<events::GuiEvent>,
  pub res_rx: tokio::sync::mpsc::Receiver<events::GuiEvent>,
  pub content_header_panel: ContentHeaderPanel,
  pub content_panel: ContentPanel,
  pub menu_panel: MenuPanel,
}

unsafe impl Send for Gui {}
impl Gui {
  // Initialize Postie with values from db
  pub async fn load_initial_data(app: PostieApi) -> (ThreadSafeState, GuiState) {
    let api = Arc::new(RwLock::new(app));

    let db_envs = Arc::clone(&api)
      .write()
      .await
      .load_environments()
      .await
      .unwrap_or(vec![EnvironmentFile::default()]);
    let saved_tabs = api.write().await.load_tabs().await.unwrap();
    let db_collections = api.write().await.load_collections().await.unwrap();
    let db_request_history_items = api
      .write()
      .await
      .load_request_response_items()
      .await
      .unwrap();
    let db_saved_requests = api.write().await.load_saved_requests().await.unwrap();
    let db_saved_responses = api.write().await.load_saved_responses().await.unwrap();
    let requests_map: HashMap<String, DBRequest> = db_saved_requests
      .into_iter()
      .map(|r| (r.id.clone(), r))
      .collect();
    let responses_map: HashMap<String, DBResponse> = db_saved_responses
      .into_iter()
      .map(|r| (r.id.clone(), r))
      .collect();
    let tabs_map: HashMap<String, Tab> = if !saved_tabs.is_empty() {
      saved_tabs
        .into_iter()
        .map(|r| (r.id.clone().to_string(), r))
        .collect()
    } else {
      let default_tab = Tab {
        id: Uuid::new_v4(),
        method: api::domain::request::HttpMethod::GET,
        url: "".into(),
        req_body: "".into(),
        req_headers: api::domain::request::RequestHeaders(vec![]),
        res_status: Some("".into()),
        res_body: "".into(),
        res_headers: api::domain::request::RequestHeaders(vec![]),
      };
      let mut default_tab_map: HashMap<String, Tab> = HashMap::new();
      default_tab_map.insert(Uuid::new_v4().to_string(), default_tab);
      default_tab_map
    };
    let collections = Arc::new(RwLock::new(db_collections));
    let environments = Arc::new(RwLock::new(db_envs.clone()));
    let requests = Arc::new(RwLock::new(requests_map));
    let request_history_items = Arc::new(RwLock::new(db_request_history_items));
    let default_active_tab = tabs_map.values().next().unwrap().clone();
    let worker_state = ThreadSafeState {
      api: Arc::clone(&api),
      collections,
      environments,
      request_history_items,
      saved_requests: Arc::clone(&requests),
      saved_responses: Arc::new(RwLock::new(responses_map)),
      tabs: Arc::new(RwLock::new(tabs_map)),
      active_tab: Arc::new(RwLock::new(default_active_tab.clone())),
      response: Arc::new(RwLock::new(Some(ResponseData::JSON(
        serde_json::from_str(&default_active_tab.res_body).unwrap_or(serde_json::Value::Null),
      )))),
      res_status: Arc::new(RwLock::new(
        default_active_tab.res_status.clone().unwrap().into(),
      )),
      oauth_response: Arc::new(RwLock::new(None)),
      received_token: Arc::new(Mutex::new(false)),
      is_requesting: Arc::new(RwLock::new(None)),
      import_result: Arc::new(Mutex::new(None)),
    };
    let gui_state = GuiState {
      url: default_active_tab.url.clone(),
      headers: Rc::new(RefCell::new(vec![])),
      active_window: RwLock::new(api::domain::ui::ActiveWindow::COLLECTIONS),
      request_window_mode: RwLock::new(api::domain::ui::RequestWindowMode::BODY),
      body_str: default_active_tab.req_body.clone(),
      env_vars: Rc::new(RefCell::new(vec![])),
      selected_http_method: default_active_tab.method.clone(),
      selected_collection: Rc::new(RefCell::new(None)),
      selected_environment: Rc::new(RefCell::new(EnvironmentFile {
        id: Uuid::new_v4().to_string(),
        name: "default".into(),
        values: Some(vec![EnvironmentValue {
          key: "HOST_URL".into(),
          value: "https://httpbin.org".into(),
          r#type: "default".into(),
          enabled: true,
        }]),
      })),
      selected_history_item: Rc::new(RefCell::new(None)),
      selected_auth_mode: api::domain::ui::AuthMode::NONE,
      selected_save_window_collection: None,
      selected_save_window_folder: None,
      selected_request: Rc::new(RefCell::new(None)),
      api_key_name: "".into(),
      api_key: "".into(),
      bearer_token: "".into(),
      oauth_token: "".into(),
      oauth_config: OAuth2Request {
        access_token_url: "".into(),
        refresh_url: "".into(),
        client_id: "".into(),
        client_secret: "".into(),
        request: OAuthRequestBody {
          grant_type: "client_credentials".into(),
          scope: "".into(),
          audience: "".into(),
        },
      },
      import_window_open: RwLock::new(false),
      new_window_open: RwLock::new(false),
      new_window_mode: RwLock::new(api::domain::ui::NewWindowMode::COLLECTION),
      save_window_open: Rc::new(RwLock::new(false)),
      new_name: "".into(),
      import_file_path: "".into(),
      import_mode: RwLock::new(api::domain::ui::ImportMode::COLLECTION),
    };
    (worker_state, gui_state)
  }

  // Spawns event worker in separate worker thread
  pub fn spawn_event_worker(
    &self,
    ctx: egui::Context,
    event_rx: tokio::sync::mpsc::Receiver<events::GuiEvent>,
    res_tx: tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    let api = self.worker_state.api.clone();
    let active_tab = self.worker_state.active_tab.clone();
    let tabs = self.worker_state.tabs.clone();
    let collections = self.worker_state.collections.clone();
    let environments = self.worker_state.environments.clone();
    let requests = self.worker_state.saved_requests.clone();
    let request_history_items = self.worker_state.request_history_items.clone();
    tokio::spawn(async move {
      Self::start_event_worker(
        event_rx,
        res_tx,
        api,
        active_tab,
        tabs,
        ctx,
        collections,
        environments,
        requests,
        request_history_items,
      )
      .await
    });
  }
  fn new(
    worker_state: ThreadSafeState,
    gui_state: GuiState,
    event_tx: tokio::sync::mpsc::Sender<events::GuiEvent>,
    res_rx: tokio::sync::mpsc::Receiver<events::GuiEvent>,
  ) -> Self {
    let content_header_panel = ContentHeaderPanel::new();
    let content_panel = ContentPanel::new();
    let menu_panel = MenuPanel::new();
    let gui = Gui {
      worker_state,
      event_tx,
      res_rx,
      gui_state,
      content_header_panel,
      content_panel,
      menu_panel,
    };
    gui
  }

  // Async event listener spawned from start_event_worker
  async fn start_event_worker(
    mut event_rx: tokio::sync::mpsc::Receiver<events::GuiEvent>,
    res_tx: tokio::sync::mpsc::Sender<events::GuiEvent>,
    api: Arc<RwLock<PostieApi>>,
    active_tab: Arc<RwLock<Tab>>,
    tabs: Arc<RwLock<HashMap<String, Tab>>>,
    ctx: egui::Context,
    collections: Arc<RwLock<Vec<Collection>>>,
    environments: Arc<RwLock<Vec<EnvironmentFile>>>,
    _requests: Arc<RwLock<HashMap<String, DBRequest>>>,
    _request_history_items: Arc<RwLock<Vec<RequestHistoryItem>>>,
  ) {
    while let Some(event) = event_rx.recv().await {
      let api_for_worker = Arc::clone(&api);
      let active_tab_for_worker = Arc::clone(&active_tab);
      let tabs_for_worker = Arc::clone(&tabs);
      let ctx_for_worker = ctx.clone();
      let res_tx_for_worker = res_tx.clone();
      let collections_for_worker = Arc::clone(&collections);
      match event {
        events::GuiEvent::SetActiveTab(input) => {
          tokio::spawn(
            async move { res_tx_for_worker.try_send(events::GuiEvent::SetActiveTab(input)) },
          );
        }
        events::GuiEvent::SubmitRequest(input) => {
          println!("handling submit request");
          tokio::spawn(async move {
            match api_for_worker
              .write()
              .await
              .make_request(PostieRequest::HTTP(input))
              .await
            {
              Ok(res) => {
                println!("Res: {:?}", res);
                let mut active_tab_writer = active_tab_for_worker.write().await;
                active_tab_writer.res_status = Some(res.status);
                active_tab_writer.res_body = res.data.to_raw_string();

                let mut tabs_writer = tabs_for_worker.write().await;
                if let Some(tab_match) = tabs_writer.get_mut(&active_tab_writer.id.to_string()) {
                  tab_match.res_status = active_tab_writer.res_status.clone();
                  tab_match.res_body = active_tab_writer.res_body.clone();
                  tab_match.url = active_tab_writer.url.clone();
                }

                ctx_for_worker.request_repaint();
              }
              Err(err) => {
                println!("Error with request: {:?}", err);
              }
            };
            res_tx_for_worker.try_send(events::GuiEvent::SetActiveTab(String::from(
              (*active_tab_for_worker.try_read().unwrap()).id,
            )))
            // TODO after response is saved, re-run db calls to refresh request/response data
          });
        }
        events::GuiEvent::SubmitOAuth2Request(data) => {
          println!("submitting oauth 2 request");
          tokio::spawn(async move {
            let res = api_for_worker
              .write()
              .await
              .make_request(PostieRequest::OAUTH(data))
              .await
              .ok()
              .unwrap();
            println!("{:?}", &res);
            let _ = Ok::<ResponseData, Error>(res.data);
          });
        }
        events::GuiEvent::RefreshCollections(_) => {
          let api_guard = api.write().await;
          match api_guard.load_collections().await {
            Ok(data) => *collections_for_worker.write().await = data,
            Err(_) => todo!(),
          }
        }
        events::GuiEvent::RefreshEnvironments() => {
          let api_guard = api.write().await;
          let envs = api_guard
            .load_environments()
            .await
            .unwrap_or(vec![EnvironmentFile {
              id: Uuid::new_v4().to_string(),
              name: String::from("default"),
              values: Some(vec![EnvironmentValue {
                key: String::from(""),
                value: String::from(""),
                r#type: String::from("default"),
                enabled: true,
              }]),
            }]);
          *environments.write().await = envs;
        }
        events::GuiEvent::RefreshRequestData(data) => {
          let request_history_items = api
            .write()
            .await
            .load_request_response_items()
            .await
            .unwrap();
          let saved_requests = api.write().await.load_saved_requests().await.unwrap();
          let saved_responses = api.write().await.load_saved_responses().await.unwrap();
          let saved_tabs = api.write().await.load_tabs().await.unwrap();
          let requests_by_id: HashMap<String, DBRequest> = saved_requests
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect();
          let responses_by_id: HashMap<String, DBResponse> = saved_responses
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect();

          let mut request_history_item_write_guard = data.request_history.try_write().unwrap();
          let mut saved_requests_write_guard = data.requests.try_write().unwrap();
          let mut saved_responses_write_guard = data.responses.try_write().unwrap();
          let mut tabs_write_guard = data.tabs.try_write().unwrap();
          *request_history_item_write_guard = request_history_items;
          *saved_requests_write_guard = requests_by_id;
          *saved_responses_write_guard = responses_by_id;
          let tabs_by_id: HashMap<Uuid, Tab> =
            saved_tabs.into_iter().map(|r| (r.id.clone(), r)).collect();
          *tabs_write_guard = tabs_by_id;
        }
        events::GuiEvent::NewCollection(data) => {
          let _blank_collection = api::domain::collection::Collection {
            info: api::domain::collection::CollectionInfo {
              id: Uuid::new_v4().to_string(),
              name: data.unwrap_or(String::from("New Collection")),
              description: None,
            },
            item: vec![],
            auth: None,
          };
        }
        events::GuiEvent::NewEnvironment(data) => {
          let _blank_env = api::domain::environment::EnvironmentFile {
            id: Uuid::new_v4().to_string(),
            name: data.unwrap_or(String::from("New Environment")),
            values: None,
          };
        }
        events::GuiEvent::RemoveTab(id) => {
          println!("removing tab {id}");
          // TODO - refresh request history after tab deletion
          api.write().await.delete_tab(id).await.unwrap();
        }
        events::GuiEvent::RemoveCollection(id) => {
          println!("removing collection {id}");
          tokio::spawn(async move {
            let mut api = api_for_worker.write().await;
            if api.delete_collection(id).await.is_ok() {
              if let Ok(new_cols) = api.load_collections().await {
                let _ =
                  res_tx_for_worker.try_send(events::GuiEvent::RefreshCollections(Some(new_cols)));
                ctx_for_worker.request_repaint();
              }
            }
          });
        }
        events::GuiEvent::RemoveCollectionFolder(data) => {
          println!("removing collection folder");
          tokio::spawn(async move {
            let mut api = api_for_worker.write().await;
            if api
              .delete_collection_folder(data.id, data.name)
              .await
              .is_ok()
            {
              if let Ok(new_cols) = api.load_collections().await {
                let _ =
                  res_tx_for_worker.try_send(events::GuiEvent::RefreshCollections(Some(new_cols)));
                ctx_for_worker.request_repaint();
              }
            }
          });
        }
        _ => {
          println!("unknown event");
        }
      }
    }
  }

  fn _refresh_request_data(
    &mut self,
    request_history: Arc<tokio::sync::RwLock<Vec<RequestHistoryItem>>>,
    responses: Arc<tokio::sync::RwLock<HashMap<String, DBResponse>>>,
    requests: Arc<tokio::sync::RwLock<HashMap<String, DBRequest>>>,
    tabs: Arc<tokio::sync::RwLock<HashMap<Uuid, Tab>>>,
  ) {
    self
      .event_tx
      .try_send(events::GuiEvent::RefreshRequestData(
        events::RefreshRequestDataPayload {
          request_history,
          responses,
          requests,
          tabs,
        },
      ))
      .unwrap();
  }

  fn refresh_collections(tx: &tokio::sync::mpsc::Sender<events::GuiEvent>) {
    tx.try_send(events::GuiEvent::RefreshCollections(None))
      .unwrap();
  }

  fn set_active_tab(&mut self, id: &str) {
    let tabs = self.worker_state.tabs.try_write().unwrap();
    let active_tab = tabs.get(id).unwrap().clone();
    let mut active_tab_write_guard = self.worker_state.active_tab.try_write().unwrap();
    *active_tab_write_guard = active_tab;
  }

  fn set_gui_values_from_active_tab(&mut self) {
    let active_tab = self.worker_state.active_tab.try_read().unwrap();
    println!("Active Tab: {:?}", active_tab);
    self.gui_state.url = active_tab.url.clone();
    self.gui_state.body_str = active_tab.req_body.clone();
    self.gui_state.selected_http_method = active_tab.method.clone();
    self.worker_state.res_status = Arc::new(RwLock::new(
      active_tab.res_status.clone().unwrap_or("".into()),
    ));
    println!("setting res body to {:?}", &active_tab.res_body);
    let response_data = ResponseData::JSON(
      serde_json::from_str(&active_tab.res_body).unwrap_or(serde_json::Value::Null),
    );
    self.worker_state.response = Arc::new(RwLock::new(Some(response_data)));
  }
}

impl App for Gui {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    while let Ok(event) = self.res_rx.try_recv() {
      match event {
        events::GuiEvent::SetActiveTab(tab_id) => {
          println!("setting gui values from active tab");
          self.set_active_tab(&tab_id);
          self.set_gui_values_from_active_tab();
          ctx.request_repaint();
        }
        events::GuiEvent::RefreshCollections(data) => {
          let cols = data.unwrap();
          if let Ok(mut cols_lock) = self.worker_state.collections.try_write() {
            *cols_lock = cols;
          };
        }
        _ => {}
      }
    }
    self
      .menu_panel
      .show(ctx, &self.event_tx, &self.gui_state, &self.worker_state);
    side_panel(self, ctx);
    self.content_header_panel.show(
      ctx,
      &self.event_tx,
      self.worker_state.active_tab.clone(),
      self.gui_state.selected_environment.clone(),
      self.gui_state.headers.clone(),
      self.gui_state.selected_auth_mode.clone(),
      self.gui_state.api_key_name.clone(),
      self.gui_state.api_key.clone(),
      self.gui_state.bearer_token.clone(),
      self.gui_state.oauth_token.clone(),
    );
    content_side_panel(self, ctx);
    self
      .content_panel
      .show(ctx, &self.gui_state, &self.worker_state, &self.event_tx);
    import_modal(self, ctx);
    new_modal(self, ctx);
    save_window(self, ctx);
  }
}

#[tokio::main]
async fn main() {
  let app = PostieApi::new().await;
  let (req_tx, req_rx) = tokio::sync::mpsc::channel(32);
  let (res_tx, res_rx) = tokio::sync::mpsc::channel(32);
  let (worker_state, gui_state) = Gui::load_initial_data(app).await;
  let native_options = NativeOptions::default();
  let _ = eframe::run_native(
    "Postie",
    native_options,
    Box::new(|cc| {
      let app = Gui::new(worker_state, gui_state, req_tx, res_rx);
      app.spawn_event_worker(cc.egui_ctx.clone(), req_rx, res_tx);
      Ok(Box::new(app))
    }),
  );
}
