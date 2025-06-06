pub mod components;

use api::{
  domain::{
    collection::Collection,
    environment::{EnvironmentFile, EnvironmentValue},
    request::{DBRequest, HttpRequest, OAuth2Request, OAuthRequestBody, PostieRequest},
    request_item::RequestHistoryItem,
    response::{DBResponse, Response, ResponseData},
    tab::Tab,
  },
  PostieApi,
};
use components::{
  content_header_panel::content_header_panel, content_panel::content_panel,
  content_side_panel::content_side_panel, import_modal::import_modal, menu_panel::menu_panel,
  new_modal::new_modal, save_window::save_window, side_panel::side_panel,
};
use eframe::{egui, App, NativeOptions};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
  sync::{Arc, Mutex},
};
use tokio::sync::RwLock;
use uuid::Uuid;

/* Holds all ui state, I found that keeping each property separate makes updating easier as you
 * dont need to worry about passing around a single reference to all parts of the ui that are
 * accessing it at once. Up for thoughts on how to make this a little cleaner though.
*/
pub struct Gui {
  pub response: Arc<RwLock<Option<ResponseData>>>,
  pub headers: Rc<RefCell<Vec<(bool, String, String)>>>,
  pub environments: Arc<RwLock<Option<Vec<api::domain::environment::EnvironmentFile>>>>,
  pub collections: Arc<RwLock<Option<Vec<api::domain::collection::Collection>>>>,
  pub request_history_items:
    Arc<RwLock<Option<Vec<api::domain::request_item::RequestHistoryItem>>>>,
  pub saved_requests: Arc<RwLock<Option<HashMap<String, api::domain::request::DBRequest>>>>,
  pub saved_responses: Arc<RwLock<Option<HashMap<String, api::domain::response::DBResponse>>>>,
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
  pub oauth_response: Arc<RwLock<Option<ResponseData>>>,
  pub oauth_config: OAuth2Request,
  pub oauth_token: String,
  pub selected_request: Rc<RefCell<Option<api::domain::collection::CollectionRequest>>>,
  pub env_vars: Rc<RefCell<Vec<EnvironmentValue>>>,
  pub active_window: RwLock<api::domain::ui::ActiveWindow>,
  pub request_window_mode: RwLock<api::domain::ui::RequestWindowMode>,
  pub url: String,
  pub body_str: String,
  pub res_status: Arc<RwLock<String>>,
  pub import_window_open: RwLock<bool>,
  pub new_window_open: RwLock<bool>,
  pub new_window_mode: RwLock<api::domain::ui::NewWindowMode>,
  pub new_name: String,
  pub save_window_open: RwLock<bool>,
  pub import_mode: RwLock<api::domain::ui::ImportMode>,
  pub import_file_path: String,
  pub import_result: Arc<Mutex<Option<String>>>,
  pub sender: tokio::sync::mpsc::Sender<Option<ResponseData>>,
  pub receiver: tokio::sync::mpsc::Receiver<Option<ResponseData>>,
  pub received_token: Arc<Mutex<bool>>,
  pub is_requesting: Arc<RwLock<Option<bool>>>,
  pub tabs: Arc<RwLock<HashMap<String, Tab>>>,
  pub active_tab: Arc<RwLock<Option<Tab>>>,
}
impl Default for Gui {
  fn default() -> Self {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    Self {
      response: Arc::new(RwLock::new(None)),
      headers: Rc::new(RefCell::new(vec![
        (true, "Content-Type".into(), "application/json".into()),
        (true, "User-Agent".into(), "postie".into()),
        (true, "Cache-Control".into(), "no-cache".into()),
      ])),
      environments: Arc::new(RwLock::new(None)),
      collections: Arc::new(RwLock::new(None)),
      env_vars: Rc::new(RefCell::new(vec![])),
      request_history_items: Arc::new(RwLock::new(None)),
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
      selected_collection: Rc::new(RefCell::new(None)),
      selected_history_item: Rc::new(RefCell::new(None)),
      selected_http_method: api::domain::request::HttpMethod::GET,
      selected_auth_mode: api::domain::ui::AuthMode::NONE,
      selected_save_window_collection: None,
      selected_save_window_folder: None,
      selected_request: Rc::new(RefCell::new(None)),
      api_key_name: "".into(),
      api_key: "".into(),
      oauth_response: Arc::new(RwLock::new(None)),
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
      bearer_token: String::from(""),
      saved_requests: Arc::new(RwLock::new(None)),
      saved_responses: Arc::new(RwLock::new(None)),
      active_window: RwLock::new(api::domain::ui::ActiveWindow::COLLECTIONS),
      request_window_mode: RwLock::new(api::domain::ui::RequestWindowMode::BODY),
      url: "".into(),
      body_str: "".into(),
      res_status: Arc::new(RwLock::new("".into())),
      import_window_open: RwLock::new(false),
      new_window_open: RwLock::new(false),
      new_window_mode: RwLock::new(api::domain::ui::NewWindowMode::COLLECTION),
      save_window_open: RwLock::new(false),
      new_name: "".into(),
      import_file_path: "".into(),
      import_mode: RwLock::new(api::domain::ui::ImportMode::COLLECTION),
      import_result: Arc::new(Mutex::new(None)),
      sender,
      receiver,
      received_token: Arc::new(Mutex::new(false)),
      is_requesting: Arc::new(RwLock::new(None)),
      tabs: Arc::new(RwLock::new(HashMap::new())),
      active_tab: Arc::new(RwLock::new(None)),
    }
  }
}
unsafe impl Send for Gui {}
impl Gui {
  async fn new() -> Self {
    // Initialize Postie with values from db
    let envs = PostieApi::load_environments()
      .await
      .unwrap_or(vec![EnvironmentFile {
        id: Uuid::new_v4().to_string(),
        name: "default".into(),
        values: Some(vec![EnvironmentValue {
          key: "".into(),
          value: "".into(),
          r#type: "default".into(),
          enabled: true,
        }]),
      }]);
    let saved_tabs = PostieApi::load_tabs().await.unwrap();
    let collections = PostieApi::load_collections().await.unwrap();
    let request_history_items = PostieApi::load_request_response_items().await.unwrap();
    let saved_requests = PostieApi::load_saved_requests().await.unwrap();
    let saved_responses = PostieApi::load_saved_responses().await.unwrap();
    let requests_by_id: HashMap<String, DBRequest> = saved_requests
      .into_iter()
      .map(|r| (r.id.clone(), r))
      .collect();
    let responses_by_id: HashMap<String, DBResponse> = saved_responses
      .into_iter()
      .map(|r| (r.id.clone(), r))
      .collect();
    let tabs_by_id: HashMap<String, Tab> = if !saved_tabs.is_empty() {
      saved_tabs.into_iter().map(|r| (r.id.clone(), r)).collect()
    } else {
      let default_tab = Tab {
        id: Uuid::new_v4().to_string(),
        method: api::domain::request::HttpMethod::GET,
        url: "".into(),
        req_body: "".into(),
        req_headers: api::domain::request::RequestHeaders(vec![]),
        res_status: None,
        res_body: "".into(),
        res_headers: api::domain::request::RequestHeaders(vec![]),
      };
      let mut default_tab_map: HashMap<String, Tab> = HashMap::new();
      default_tab_map.insert(Uuid::new_v4().to_string(), default_tab);
      default_tab_map
    };
    let mut default = Gui::default();
    default.environments = Arc::new(RwLock::from(Some(envs)));
    default.collections = Arc::new(RwLock::from(Some(collections)));
    default.request_history_items = Arc::new(RwLock::from(Some(request_history_items)));
    default.saved_requests = Arc::new(RwLock::from(Some(requests_by_id)));
    default.saved_responses = Arc::new(RwLock::from(Some(responses_by_id)));
    default.tabs = Arc::new(RwLock::new(tabs_by_id.clone()));
    let default_active_tab = tabs_by_id.values().next().unwrap().clone();
    default.active_tab = Arc::new(RwLock::new(Some(default_active_tab.clone())));
    default.url = default_active_tab.url.clone();
    default.body_str = default_active_tab.req_body.clone();
    default.selected_http_method = default_active_tab.method.clone();
    default.res_status = Arc::new(RwLock::new(
      default_active_tab.res_status.clone().unwrap_or("".into()),
    ));
    let response_data = ResponseData::JSON(
      serde_json::from_str(&default_active_tab.res_body).unwrap_or(serde_json::Value::Null),
    );
    default.response = Arc::new(RwLock::new(Some(response_data)));
    default
  }
  async fn refresh_request_data(
    request_history: Arc<RwLock<Option<Vec<RequestHistoryItem>>>>,
    responses: Arc<RwLock<Option<HashMap<String, DBResponse>>>>,
    requests: Arc<RwLock<Option<HashMap<String, DBRequest>>>>,
    tabs: Arc<RwLock<HashMap<String, Tab>>>,
  ) {
    let request_history_items = PostieApi::load_request_response_items().await.unwrap();
    let saved_requests = PostieApi::load_saved_requests().await.unwrap();
    let saved_responses = PostieApi::load_saved_responses().await.unwrap();
    let saved_tabs = PostieApi::load_tabs().await.unwrap();
    let requests_by_id: HashMap<String, DBRequest> = saved_requests
      .into_iter()
      .map(|r| (r.id.clone(), r))
      .collect();
    let responses_by_id: HashMap<String, DBResponse> = saved_responses
      .into_iter()
      .map(|r| (r.id.clone(), r))
      .collect();

    let mut request_history_item_write_guard = request_history.try_write().unwrap();
    let mut saved_requests_write_guard = requests.try_write().unwrap();
    let mut saved_responses_write_guard = responses.try_write().unwrap();
    let mut tabs_write_guard = tabs.try_write().unwrap();
    *request_history_item_write_guard = Some(request_history_items);
    *saved_requests_write_guard = Some(requests_by_id);
    *saved_responses_write_guard = Some(responses_by_id);
    let tabs_by_id = saved_tabs.into_iter().map(|r| (r.id.clone(), r)).collect();
    *tabs_write_guard = tabs_by_id;
  }
  async fn refresh_collections(old_collections: Arc<RwLock<Option<Vec<Collection>>>>) {
    let collections = PostieApi::load_collections().await.unwrap();
    let mut collection_write_guard = old_collections.try_write().unwrap();
    *collection_write_guard = Some(collections);
  }
  async fn refresh_environments(old_environments: Arc<RwLock<Option<Vec<EnvironmentFile>>>>) {
    let envs = PostieApi::load_environments()
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
    let mut environment_write_guard = old_environments.try_write().unwrap();
    *environment_write_guard = Some(envs);
  }
  async fn submit(input: HttpRequest) -> anyhow::Result<Response> {
    PostieApi::make_request(PostieRequest::HTTP(input)).await
  }
  // egui needs to run on the main thread so all async requests need to be run on a worker
  // thread.
  fn spawn_submit(&mut self, input: HttpRequest) -> anyhow::Result<()> {
    // TODO figure out how to impl Send for Gui so it can be passed to another thread.
    // currently getting an error. Workaround is to just clone the PostieApi
    let response_for_worker = self.response.clone();
    let request_history_for_worker = self.request_history_items.clone();
    let saved_requests_for_worker = self.saved_requests.clone();
    let saved_response_for_worker = self.saved_responses.clone();
    let is_requesting_for_worker = self.is_requesting.clone();
    let res_status_for_worker = self.res_status.clone();
    let tabs_for_worker = self.tabs.clone();
    tokio::spawn(async move {
      let mut result_write_guard = response_for_worker.try_write().unwrap();
      let mut is_requesting_write_guard = is_requesting_for_worker.try_write().unwrap();
      let mut res_status_write_guard = res_status_for_worker.try_write().unwrap();
      // TODO - figure out why ui doesnt recognize when set to true, only when request is
      // complete and set to false.
      *is_requesting_write_guard = Some(true);

      match Self::submit(input).await {
        Ok(res) => {
          println!("Res: {:?}", res);
          *result_write_guard = Some(res.data);
          *res_status_write_guard = res.status;
          *is_requesting_write_guard = Some(false);
        }
        Err(err) => {
          println!("Error with request: {:?}", err);
          *result_write_guard = None;
          *is_requesting_write_guard = Some(false);
        }
      };

      // after response is saved, re-run db calls to refresh request/response data
      Self::refresh_request_data(
        request_history_for_worker,
        saved_response_for_worker,
        saved_requests_for_worker,
        tabs_for_worker,
      )
      .await
    });
    Ok(())
  }
  async fn delete_tab(id: Uuid) {
    PostieApi::delete_tab(id).await.unwrap();
  }
  fn spawn_delete_tab(&mut self, id: Uuid) {
    let tabs_for_worker = self.tabs.clone();
    let saved_requests_for_worker = self.saved_requests.clone();
    let saved_response_for_worker = self.saved_responses.clone();
    let request_history_for_worker = self.request_history_items.clone();
    tokio::spawn(async move {
      Self::delete_tab(id).await;
      Self::refresh_request_data(
        request_history_for_worker,
        saved_response_for_worker,
        saved_requests_for_worker,
        tabs_for_worker,
      )
      .await;
    });
  }
  async fn oauth_token_request(input: OAuth2Request) -> anyhow::Result<ResponseData> {
    let res = PostieApi::make_request(PostieRequest::OAUTH(input))
      .await
      .ok()
      .unwrap();
    println!("{:?}", &res);
    Ok(res.data)
  }
  fn spawn_ouath_request(
    sender: &mut tokio::sync::mpsc::Sender<Option<ResponseData>>,
    input: OAuth2Request,
  ) -> anyhow::Result<()> {
    let sender_for_worker = sender.clone();
    tokio::spawn(async move {
      match Self::oauth_token_request(input).await {
        Ok(res) => {
          println!("OAuth Response: {:?}", res);
          _ = sender_for_worker.send(Some(res)).await;
        }
        Err(_err) => {
          todo!()
        }
      };
    });
    Ok(())
  }

  fn remove_duplicate_headers(headers: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut unique_keys = HashSet::new();
    let mut result = Vec::new();
    for (key, value) in headers {
      if !unique_keys.contains(&key) {
        unique_keys.insert(key.clone());
        result.push((key.clone(), value.clone()));
      }
    }
    result
  }

  fn set_active_tab(&mut self, id: &str) {
    let tabs = self.tabs.try_read().unwrap();
    let active_tab = tabs.get(id).unwrap().clone();
    let mut active_tab_write_guard = self.active_tab.try_write().unwrap();
    *active_tab_write_guard = Some(active_tab);
  }

  fn set_gui_values_from_active_tab(&mut self) {
    let active_tab = self.active_tab.try_read().unwrap();
    println!("Active Tab: {:?}", active_tab);
    if let Some(active_tab) = active_tab.as_ref() {
      self.url = active_tab.url.clone();
      self.body_str = active_tab.req_body.clone();
      self.selected_http_method = active_tab.method.clone();
      self.res_status = Arc::new(RwLock::new(
        active_tab.res_status.clone().unwrap_or("".into()),
      ));
      let response_data = ResponseData::JSON(
        serde_json::from_str(&active_tab.res_body).unwrap_or(serde_json::Value::Null),
      );
      self.response = Arc::new(RwLock::new(Some(response_data)));
    }
  }
}

impl App for Gui {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    menu_panel(self, ctx);
    side_panel(self, ctx);
    content_header_panel(self, ctx);
    content_side_panel(self, ctx);
    content_panel(self, ctx);
    import_modal(self, ctx);
    new_modal(self, ctx);
    save_window(self, ctx);
  }
}

#[tokio::main]
async fn main() {
  let app = Gui::new().await;
  let native_options = NativeOptions::default();
  let _ = eframe::run_native("Postie", native_options, Box::new(|_cc| Box::new(app)));
}
