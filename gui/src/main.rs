use api::{
    domain::{
        environment::{EnvironmentFile, EnvironmentValue},
        request::DBRequest,
        response::DBResponse,
    },
    HttpMethod, HttpRequest, PostieApi,
};
use eframe::{
    egui::{CentralPanel, ComboBox, ScrollArea, SidePanel, TextEdit, TopBottomPanel},
    App, NativeOptions,
};
use egui::TextStyle;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cell::{RefCell, RefMut},
    collections::{HashMap, HashSet},
    error::Error,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum ActiveWindow {
    REQUEST,
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
    pub response: Arc<RwLock<Option<Value>>>,
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
            selected_environment: Rc::new(RefCell::new(EnvironmentFile {
                id: Uuid::new_v4().to_string(),
                name: String::from("default"),
                values: Some(vec![EnvironmentValue { key: String::from("HOST_URL"), value: String::from("https://httpbin.org/json"), r#type: String::from("default"), enabled: true }]),
            })),
            selected_collection: Rc::new(RefCell::new(None)),
            environments: Rc::new(RefCell::new(None)),
            collections: Rc::new(RefCell::new(None)),
            env_vars: Rc::new(RefCell::new(vec![])),
            request_history_items: Rc::new(RefCell::new(None)),
            selected_history_item: Rc::new(RefCell::new(None)),
            saved_requests: Rc::new(RefCell::new(None)),
            saved_responses: Rc::new(RefCell::new(None)),
            active_window: RwLock::new(ActiveWindow::REQUEST),
            request_window_mode: RwLock::new(RequestWindowMode::BODY),
            selected_http_method: HttpMethod::GET,
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
                values: Some(vec![EnvironmentValue { key: String::from(""), value: String::from(""), r#type: String::from("default"), enabled: true }]),
            }]);
        let collections = PostieApi::load_collections()
            .await
            .unwrap();
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
    async fn submit(input: HttpRequest) -> Result<Value, Box<dyn Error>> {
        PostieApi::make_request(input).await
    }
    fn spawn_submit(&mut self, input: HttpRequest) -> Result<(), Box<dyn Error>> {
        // TODO figure out how to impl Send for Gui so it can be passed to another thread.
        // currently getting an error. Workaround is to just clone the PostieApi
        let result_for_worker = self.response.clone();
        tokio::spawn(async move {
            match Gui::submit(input).await {
                Ok(res) => {
                    println!("Res: {}", res);
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
        TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Menu", |ui| {
                    ui.menu_button("New", |ui| {
                        if ui.button("Collection").clicked() {
                            ui.close_menu();
                        };
                        if ui.button("Evnironment").clicked() {
                            ui.close_menu();
                        };
                    });
                    ui.menu_button("Import", |ui| {
                        if ui.button("Collection").clicked() {
                            if let Ok(mut import_open) = self.import_window_open.try_write() {
                                *import_open = true;
                            }
                            if let Ok(mut import_mode) = self.import_mode.try_write() {
                                *import_mode = ImportMode::COLLECTION;
                            }
                        };
                        if ui.button("Environment").clicked() {
                            if let Ok(mut import_open) = self.import_window_open.try_write() {
                                *import_open = true;
                            }
                            if let Ok(mut import_mode) = self.import_mode.try_write() {
                                *import_mode = ImportMode::ENVIRONMENT;
                            }
                        };
                    });
                    ui.menu_button("Export", |ui| {
                        if ui.button("Collection").clicked() {
                            ui.close_menu();
                        };
                        if ui.button("Environment").clicked() {
                            ui.close_menu();
                        };
                    });
                });
            });
        });
        SidePanel::left("nav_panel").show(ctx, |ui| {
            if let Ok(mut active_window) = self.active_window.try_write() {
                if ui.button("Request").clicked() {
                    *active_window = ActiveWindow::REQUEST;
                }
                if ui.button("Environment").clicked() {
                    *active_window = ActiveWindow::ENVIRONMENT;
                }
                if ui.button("History").clicked() {
                    *active_window = ActiveWindow::HISTORY;
                }
            }
        });
        if let Ok(active_window) = self.active_window.try_read() {
            SidePanel::left("content_panel").show(ctx, |ui| match *active_window {
                ActiveWindow::REQUEST => {
                    ui.label("Collections");
                    let collections_clone = Rc::clone(&self.collections);
                    let collections = collections_clone.borrow();
                    if let Some(cols) = &*collections {
                        for c in cols {
                            ui.selectable_value(
                                &mut self.selected_collection,
                                Rc::new(RefCell::from(Some(c.clone()))),
                                format!("{}", c.info.name)
                            );
                        }
                    }
                }
                ActiveWindow::ENVIRONMENT => {
                    ui.label("Environments");
                    let envs_clone = Rc::clone(&self.environments);
                    let envs = envs_clone.borrow();
                    if let Some(env_vec) = &*envs {
                        for env in env_vec {
                            ui.selectable_value(
                                &mut self.selected_environment,
                                Rc::new(RefCell::from(env.clone())),
                                format!("{}", env.name),
                            );
                        }
                    }
                }
                ActiveWindow::HISTORY => {
                    ui.label("History");
                    let history_items_clone = Rc::clone(&self.request_history_items);
                    let history_items = history_items_clone.borrow();
                    if let Some(item_vec) = &*history_items {
                        for item in item_vec {
                            if ui
                                .selectable_value(
                                    &mut self.selected_history_item,
                                    Rc::new(RefCell::from(Some(item.clone()))),
                                    format!("{}", item.id),
                                )
                                .clicked()
                            {
                                // TODO - replace url, method, request body, response body
                                let requests_clone = self.saved_requests.borrow();
                                let responses_clone = self.saved_responses.borrow();
                                let requests = requests_clone.as_ref().unwrap();
                                let responses = responses_clone.as_ref().unwrap();
                                let historical_request = requests.get(&item.request_id).unwrap();
                                let historical_response = responses.get(&item.response_id).unwrap();
                                self.url = historical_request.url.clone();
                                self.selected_http_method =
                                    HttpMethod::from_str(&historical_request.method).unwrap();
                                match &historical_request.body {
                                    Some(body_json) => {
                                        let body_str =
                                            serde_json::from_value(body_json.clone()).unwrap();
                                        self.body_str = body_str;
                                    }
                                    None => self.body_str = String::from(""),
                                }
                                let ui_response_clone = self.response.clone();
                                let mut ui_response_guard = ui_response_clone.try_write().unwrap();
                                let response_body = &historical_response.body;
                                match response_body {
                                    Some(body) => *ui_response_guard = Some(body.clone()),
                                    None => *ui_response_guard = None,
                                }
                            }
                        }
                    }
                }
            });
        }
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Welcome to Postie!");
            ui.horizontal(|ui| {
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", &mut self.selected_http_method))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::GET, "GET");
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::POST, "POST");
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::PUT, "PUT");
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::DELETE, "DELETE");
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::PATCH, "PATCH");
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::OPTIONS, "OPTIONS");
                        ui.selectable_value(&mut self.selected_http_method, HttpMethod::HEAD, "HEAD");
                    });
                ui.label("URL:");
                ui.text_edit_singleline(&mut self.url);
                if ui.button("Submit").clicked() {
                    let body = if self.selected_http_method != HttpMethod::GET {
                        Some(serde_json::from_str(&self.body_str).expect("Body is invalid json"))
                    } else {
                        None
                    };
                    let submitted_headers = self
                        .headers
                        .borrow_mut()
                        .iter()
                        .filter(|h| h.0 == true)
                        .map(|h| (h.1.to_owned(), h.2.to_owned()))
                        .collect();
                    let request = HttpRequest {
                        id: Uuid::new_v4(),
                        name: None,
                        headers: Some(Gui::remove_duplicate_headers(submitted_headers)),
                        body,
                        method: self.selected_http_method.clone(),
                        url: self.url.clone(),
                        environment: self.selected_environment.borrow().clone(),
                    };

                    let _ = Gui::spawn_submit(self, request);
                }
            });
            if let Ok(mut request_window_mode) = self.request_window_mode.try_write() {
                ui.horizontal(|ui| {
                    if ui.button("Environment").clicked() {
                        *request_window_mode = RequestWindowMode::ENVIRONMENT;
                    }
                    if ui.button("Params").clicked() {
                        *request_window_mode = RequestWindowMode::PARAMS;
                    }
                    if ui.button("Headers").clicked() {
                        *request_window_mode = RequestWindowMode::HEADERS;
                    }
                    if ui.button("Body").clicked() {
                        *request_window_mode = RequestWindowMode::BODY;
                    }
                });
            }
        });
        if let Ok(request_window_mode) = self.request_window_mode.try_read() {
            match *request_window_mode {
                RequestWindowMode::BODY => {
                    TopBottomPanel::top("request_panel")
                        .resizable(true)
                        .min_height(250.0)
                        .show(ctx, |ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    TextEdit::multiline(&mut self.body_str)
                                        .code_editor()
                                        .desired_rows(20)
                                        .lock_focus(true)
                                        .font(TextStyle::Monospace),
                                );
                            });
                        });
                    if self.response.try_read().unwrap().is_some() {
                        CentralPanel::default().show(ctx, |ui| {
                            ui.label(
                                self.response
                                    .try_read()
                                    .unwrap()
                                    .as_ref()
                                    .unwrap()
                                    .to_string(),
                            );
                        });
                    }
                }
                RequestWindowMode::PARAMS => {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.label("params");
                    });
                }
                RequestWindowMode::HEADERS => {
                    CentralPanel::default().show(ctx, |ui| {
                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::auto())
                            .column(Column::auto())
                            .column(Column::auto());
                        table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Enabled");
                                });
                                header.col(|ui| {
                                    ui.strong("Key");
                                });
                                header.col(|ui| {
                                    ui.strong("Value");
                                });
                            })
                            .body(|mut body| {
                                for header in self.headers.borrow_mut().iter_mut() {
                                    body.row(30.0, |mut row| {
                                        let (enabled, key, value) = header;
                                        row.col(|ui| {
                                            ui.checkbox(enabled, "");
                                        });
                                        row.col(|ui| {
                                            ui.text_edit_singleline(key);
                                        });
                                        row.col(|ui| {
                                            ui.text_edit_singleline(value);
                                        });
                                    });
                                }
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        if ui.button("Add").clicked() {
                                            self.headers.borrow_mut().push((
                                                true,
                                                String::from(""),
                                                String::from(""),
                                            ));
                                        };
                                    });
                                });
                            });
                    });
                }
                RequestWindowMode::ENVIRONMENT => {
                    CentralPanel::default().show(ctx, |ui| {
                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::auto())
                            .column(Column::auto())
                            .column(Column::auto())
                            .column(Column::auto());
                        table
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Enabled");
                                });
                                header.col(|ui| {
                                    ui.strong("Key");
                                });
                                header.col(|ui| {
                                    ui.strong("Value");
                                });
                                header.col(|ui| {
                                    ui.strong("Type");
                                });
                            })
                            .body(|mut body| {
                                let selected_environment =
                                    self.selected_environment.borrow_mut();
                                    let mut values_ref = RefMut::map(selected_environment, |env| &mut env.values);
                                    if let Some(values) = values_ref.as_mut() {
                                        for env_var in values {
                                            body.row(30.0, |mut row| {
                                                row.col(|ui| {
                                                    ui.checkbox(&mut env_var.enabled, "");
                                                });
                                                row.col(|ui| {
                                                    ui.text_edit_singleline(&mut env_var.key);
                                                });
                                                row.col(|ui| {
                                                    ui.text_edit_singleline(&mut env_var.value);
                                                });
                                                row.col(|ui| {
                                                    ui.text_edit_singleline(&mut env_var.r#type);
                                                });
                                            });
                                        }
                                }
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        if ui.button("Add").clicked() {
                                            if let Some(vals) = values_ref.as_mut() {
                                                vals.push(EnvironmentValue {
                                                    key: String::from(""),
                                                    value: String::from(""),
                                                    r#type: String::from("default"),
                                                    enabled: true,
                                                });

                                            }
                                        };
                                    });
                                });
                            });
                    });
                }
            };
        }
        if let Ok(mut import_window_open) = self.import_window_open.try_write() {
            if *import_window_open == true {
                egui::Window::new("Import Modal")
                    .open(&mut *import_window_open)
                    .show(ctx, |ui| {
                        ui.label("Please copy and paste the file path to import");
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.import_file_path);
                            if ui.button("Import").clicked() {
                                let path = self.import_file_path.to_owned();
                                if let Ok(import_mode) = self.import_mode.try_read() {
                                    let import_result_clone = self.import_result.clone();
                                    match *import_mode {
                                        ImportMode::COLLECTION => {
                                            tokio::spawn(async move {
                                                PostieApi::import_collection(&path).await
                                            });
                                        }
                                        ImportMode::ENVIRONMENT => {
                                            tokio::spawn(async move {
                                                let res = PostieApi::import_environment(&path)
                                                    .await
                                                    .unwrap();
                                                let mut data = import_result_clone.lock().unwrap();
                                                *data = Some(res);
                                            });
                                        }
                                    };
                                }
                            };
                            let i = self.import_result.lock().unwrap();
                            if let Some(import_res) = &*i {
                                ui.label(import_res);
                            }
                        });
                    });
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Gui::new().await;
    let native_options = NativeOptions::default();
    let _ = eframe::run_native("Postie", native_options, Box::new(|_cc| Box::new(app)));
}
