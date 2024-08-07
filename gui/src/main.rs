pub mod components;

use anyhow;
use api::{
    domain::{
        collection::Collection,
        environment::{EnvironmentFile, EnvironmentValue},
        request::DBRequest,
        request_item::RequestHistoryItem,
        response::DBResponse,
    },
    PostieApi, ResponseData,
};
use components::{
    content_header_panel::content_header_panel, content_panel::content_panel,
    content_side_panel::content_side_panel, import_modal::import_modal, menu_panel::menu_panel,
    side_panel::side_panel,
};
use eframe::{egui, App, NativeOptions};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum ActiveWindow {
    COLLECTIONS,
    ENVIRONMENT,
    HISTORY,
}
#[derive(Serialize, Deserialize)]
pub enum ImportMode {
    COLLECTION,
    ENVIRONMENT,
}
#[derive(Serialize, Deserialize)]
pub enum RequestWindowMode {
    AUTHORIZATION,
    PARAMS,
    HEADERS,
    BODY,
    ENVIRONMENT,
}

#[derive(Debug, PartialEq)]
pub enum AuthMode {
    APIKEY,
    BEARER,
    OAUTH2,
    NONE,
}
impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/* Holds all ui state, I found that keeping each property separate makes updating easier as you
 * dont need to worry about passing around a single reference to all parts of the ui that are
 * accessing it at once. Up for thoughts on how to make this a little cleaner though.
*/
pub struct Gui {
    pub response: Arc<RwLock<Option<api::ResponseData>>>,
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
    pub selected_http_method: api::HttpMethod,
    pub selected_auth_mode: AuthMode,
    pub api_key: String,
    pub api_key_name: String,
    pub bearer_token: String,
    pub oauth_response: Arc<RwLock<Option<ResponseData>>>,
    pub oauth_config: api::OAuth2Request,
    pub oauth_token: String,
    pub selected_request: Rc<RefCell<Option<api::domain::collection::CollectionRequest>>>,
    pub env_vars: Rc<RefCell<Vec<EnvironmentValue>>>,
    pub active_window: RwLock<ActiveWindow>,
    pub request_window_mode: RwLock<RequestWindowMode>,
    pub url: String,
    pub body_str: String,
    pub import_window_open: RwLock<bool>,
    pub import_mode: RwLock<ImportMode>,
    pub import_file_path: String,
    pub import_result: Arc<Mutex<Option<String>>>,
    pub sender: tokio::sync::mpsc::Sender<Option<ResponseData>>,
    pub receiver: tokio::sync::mpsc::Receiver<Option<ResponseData>>,
    pub received_token: Arc<Mutex<bool>>,
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
            selected_http_method: api::HttpMethod::GET,
            selected_auth_mode: AuthMode::NONE,
            selected_request: Rc::new(RefCell::new(None)),
            api_key_name: "".into(),
            api_key: "".into(),
            oauth_response: Arc::new(RwLock::new(None)),
            oauth_token: "".into(),
            oauth_config: api::OAuth2Request {
                access_token_url: "".into(),
                refresh_url: "".into(),
                client_id: "".into(),
                client_secret: "".into(),
                request: api::OAuthRequestBody {
                    grant_type: "client_credentials".into(),
                    scope: "".into(),
                    audience: "".into(),
                },
            },
            bearer_token: String::from(""),
            saved_requests: Arc::new(RwLock::new(None)),
            saved_responses: Arc::new(RwLock::new(None)),
            active_window: RwLock::new(ActiveWindow::COLLECTIONS),
            request_window_mode: RwLock::new(RequestWindowMode::BODY),
            url: String::from("{{HOST_URL}}/json"),
            body_str: "{ \"foo\": \"bar\" }".into(),
            import_window_open: RwLock::new(false),
            import_file_path: "".into(),
            import_mode: RwLock::new(ImportMode::COLLECTION),
            import_result: Arc::new(Mutex::new(None)),
            sender,
            receiver,
            received_token: Arc::new(Mutex::new(false)),
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
        let mut default = Gui::default();
        default.environments = Arc::new(RwLock::from(Some(envs)));
        default.collections = Arc::new(RwLock::from(Some(collections)));
        default.request_history_items = Arc::new(RwLock::from(Some(request_history_items)));
        default.saved_requests = Arc::new(RwLock::from(Some(requests_by_id)));
        default.saved_responses = Arc::new(RwLock::from(Some(responses_by_id)));
        default
    }
    async fn refresh_request_data(
        request_history: Arc<RwLock<Option<Vec<RequestHistoryItem>>>>,
        responses: Arc<RwLock<Option<HashMap<String, DBResponse>>>>,
        requests: Arc<RwLock<Option<HashMap<String, DBRequest>>>>,
    ) {
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

        let mut request_history_item_write_guard = request_history.try_write().unwrap();
        let mut saved_requests_write_guard = requests.try_write().unwrap();
        let mut saved_responses_write_guard = responses.try_write().unwrap();
        *request_history_item_write_guard = Some(request_history_items).into();
        *saved_requests_write_guard = Some(requests_by_id);
        *saved_responses_write_guard = Some(responses_by_id).into();
    }
    async fn refresh_collections(old_collections: Arc<RwLock<Option<Vec<Collection>>>>) {
        let collections = PostieApi::load_collections().await.unwrap();
        let mut collection_write_guard = old_collections.try_write().unwrap();
        *collection_write_guard = Some(collections).into();
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
        *environment_write_guard = Some(envs).into();
    }
    async fn submit(input: api::HttpRequest) -> anyhow::Result<api::ResponseData> {
        PostieApi::make_request(api::PostieRequest::HTTP(input)).await
    }
    // egui needs to run on the main thread so all async requests need to be run on a worker
    // thread.
    fn spawn_submit(&mut self, input: api::HttpRequest) -> anyhow::Result<()> {
        // TODO figure out how to impl Send for Gui so it can be passed to another thread.
        // currently getting an error. Workaround is to just clone the PostieApi
        let response_for_worker = self.response.clone();
        let request_history_for_worker = self.request_history_items.clone();
        let saved_requests_for_worker = self.saved_requests.clone();
        let saved_response_for_worker = self.saved_responses.clone();
        tokio::spawn(async move {
            let mut result_write_guard = response_for_worker.try_write().unwrap();
            match Self::submit(input).await {
                Ok(res) => {
                    println!("Res: {:?}", res);
                    *result_write_guard = Some(res);
                }
                Err(err) => {
                    println!("Error with request: {:?}", err);
                    *result_write_guard = None;
                }
            };

            // after response is saved, re-run db calls to refresh request/response data
            Self::refresh_request_data(
                request_history_for_worker,
                saved_response_for_worker,
                saved_requests_for_worker,
            )
            .await
        });
        Ok(())
    }
    async fn oauth_token_request(input: api::OAuth2Request) -> anyhow::Result<api::ResponseData> {
        let res = PostieApi::make_request(api::PostieRequest::OAUTH(input))
            .await
            .ok()
            .unwrap();
        println!("{:?}", &res);
        Ok(res)
    }
    fn spawn_ouath_request(
        sender: &mut tokio::sync::mpsc::Sender<Option<ResponseData>>,
        input: api::OAuth2Request,
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
}

impl App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        menu_panel(self, ctx);
        side_panel(self, ctx);
        content_header_panel(self, ctx);
        content_side_panel(self, ctx);
        content_panel(self, ctx);
        import_modal(self, ctx);
    }
}

#[tokio::main]
async fn main() {
    let app = Gui::new().await;
    let native_options = NativeOptions::default();
    let _ = eframe::run_native("Postie", native_options, Box::new(|_cc| Box::new(app)));
}
