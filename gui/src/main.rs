pub mod components;

use api::{
    domain::{
        environment::{EnvironmentFile, EnvironmentValue},
        request::DBRequest,
        response::DBResponse,
    },
    HttpMethod, HttpRequest, PostieApi, ResponseData,
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
    error::Error,
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
    PARAMS,
    HEADERS,
    BODY,
    ENVIRONMENT,
}

pub struct Gui {
    pub response: Arc<RwLock<Option<ResponseData>>>,
    pub headers: Rc<RefCell<Vec<(bool, String, String)>>>,
    pub environments: Rc<RefCell<Option<Vec<api::domain::environment::EnvironmentFile>>>>,
    pub collections: Rc<RefCell<Option<Vec<api::domain::collection::Collection>>>>,
    pub request_history_items:
        Rc<RefCell<Option<Vec<api::domain::request_item::RequestHistoryItem>>>>,
    pub saved_requests: Rc<RefCell<Option<HashMap<String, api::domain::request::DBRequest>>>>,
    pub saved_responses: Rc<RefCell<Option<HashMap<String, api::domain::response::DBResponse>>>>,
    pub selected_history_item: Rc<RefCell<Option<api::domain::request_item::RequestHistoryItem>>>,
    pub selected_environment: Rc<RefCell<api::domain::environment::EnvironmentFile>>,
    pub selected_collection: Rc<RefCell<Option<api::domain::collection::Collection>>>,
    pub selected_http_method: HttpMethod,
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
}
impl Default for Gui {
    fn default() -> Self {
        Self {
            response: Arc::new(RwLock::new(None)),
            headers: Rc::new(RefCell::new(vec![
                (
                    true,
                    String::from("Content-Type"),
                    String::from("application/json"),
                ),
                (true, String::from("User-Agent"), String::from("postie")),
                (
                    true,
                    String::from("Cache-Control"),
                    String::from("no-cache"),
                ),
            ])),
            environments: Rc::new(RefCell::new(None)),
            collections: Rc::new(RefCell::new(None)),
            env_vars: Rc::new(RefCell::new(vec![])),
            request_history_items: Rc::new(RefCell::new(None)),
            selected_environment: Rc::new(RefCell::new(EnvironmentFile {
                id: Uuid::new_v4().to_string(),
                name: String::from("default"),
                values: Some(vec![EnvironmentValue {
                    key: String::from("HOST_URL"),
                    value: String::from("https://httpbin.org"),
                    r#type: String::from("default"),
                    enabled: true,
                }]),
            })),
            selected_collection: Rc::new(RefCell::new(None)),
            selected_history_item: Rc::new(RefCell::new(None)),
            selected_http_method: HttpMethod::GET,
            selected_request: Rc::new(RefCell::new(None)),
            saved_requests: Rc::new(RefCell::new(None)),
            saved_responses: Rc::new(RefCell::new(None)),
            active_window: RwLock::new(ActiveWindow::COLLECTIONS),
            request_window_mode: RwLock::new(RequestWindowMode::BODY),
            url: String::from("{{HOST_URL}}/json"),
            body_str: String::from("{ \"foo\": \"bar\" }"),
            import_window_open: RwLock::new(false),
            import_file_path: String::from(""),
            import_mode: RwLock::new(ImportMode::COLLECTION),
            import_result: Arc::new(Mutex::new(None)),
        }
    }
}
impl Gui {
    async fn new() -> Self {
        // Initialize Postie with values from db
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
        default.environments = Rc::new(RefCell::from(Some(envs)));
        default.collections = Rc::new(RefCell::from(Some(collections)));
        default.request_history_items = Rc::new(RefCell::from(Some(request_history_items)));
        default.saved_requests = Rc::new(RefCell::from(Some(requests_by_id)));
        default.saved_responses = Rc::new(RefCell::from(Some(responses_by_id)));
        default
    }
}
impl Gui {
    async fn submit(input: HttpRequest) -> Result<ResponseData, Box<dyn Error>> {
        PostieApi::make_request(input).await
    }
    fn spawn_submit(&mut self, input: HttpRequest) -> Result<(), Box<dyn Error>> {
        // TODO figure out how to impl Send for Gui so it can be passed to another thread.
        // currently getting an error. Workaround is to just clone the PostieApi
        let result_for_worker = self.response.clone();
        tokio::spawn(async move {
            match Gui::submit(input).await {
                Ok(res) => {
                    println!("Res: {:?}", res);
                    let mut result_write_guard = result_for_worker.try_write().unwrap();
                    *result_write_guard = Some(res);
                }
                Err(err) => {
                    println!("Error with request: {:?}", err);
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
