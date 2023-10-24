use api::{
    domain::environment::{EnvironmentFile, EnvironmentValue},
    initialize_db, HttpMethod, HttpRequest, PostieApi,
};
use eframe::{
    egui::{CentralPanel, ComboBox, ScrollArea, SidePanel, TextEdit, TopBottomPanel},
    App, NativeOptions,
};
use egui::TextStyle;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cell::RefCell, collections::HashSet, error::Error, rc::Rc, sync::Arc};
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

#[derive(Serialize, Deserialize)]
pub struct GuiConfig {
    pub active_window: ActiveWindow,
    pub request_window_mode: RequestWindowMode,
    pub selected_http_method: HttpMethod,
    pub url: String,
    pub body_str: String,
    pub import_window_open: bool,
    pub import_mode: ImportMode,
    pub import_file_path: String,
}
impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            active_window: ActiveWindow::REQUEST,
            request_window_mode: RequestWindowMode::BODY,
            selected_http_method: HttpMethod::GET,
            url: String::from("https://httpbin.org/json"),
            body_str: String::from("{ \"foo\": \"bar\" }"),
            import_window_open: false,
            import_file_path: String::from(""),
            import_mode: ImportMode::COLLECTION,
        }
    }
}

pub struct Gui {
    pub api: PostieApi,
    pub config: Arc<RwLock<GuiConfig>>,
    pub response: Arc<RwLock<Option<Value>>>,
    pub headers: Rc<RefCell<Vec<(bool, String, String)>>>,
    pub environment: Rc<RefCell<Option<api::domain::environment::EnvironmentFile>>>,
}
impl Default for Gui {
    fn default() -> Self {
        Self {
            api: PostieApi::new(),
            config: Arc::new(RwLock::new(GuiConfig::default())),
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
            environment: Rc::new(RefCell::new(Some(EnvironmentFile {
                id: String::from("id"),
                name: String::from("some environment"),
                values: Some(vec![EnvironmentValue {
                    key: String::from("HOST_URL"),
                    value: String::from("https://httpbin.org"),
                    r#type: String::from("default"),
                    enabled: true,
                }]),
            }))),
        }
    }
}
impl Gui {
    async fn submit(client: &PostieApi, input: HttpRequest) -> Result<Value, Box<dyn Error>> {
        PostieApi::make_request(client, input).await
    }
    fn spawn_submit(&self, input: HttpRequest) -> Result<(), Box<dyn Error>> {
        // TODO figure out how to imple Send for Gui so it can be passed to another thread.
        // currently getting an error. Workaround is to just clone the PostieApi
        let result_for_worker = self.response.clone();
        let client_clone = self.api.clone();
        tokio::spawn(async move {
            match Gui::submit(&client_clone, input).await {
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
                            if let Ok(mut config) = self.config.try_write() {
                                (*config).import_window_open = true;
                                (*config).import_mode = ImportMode::COLLECTION;
                            }
                        };
                        if ui.button("Environment").clicked() {
                            if let Ok(mut config) = self.config.try_write() {
                                (*config).import_window_open = true;
                                (*config).import_mode = ImportMode::ENVIRONMENT;
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
            if let Ok(mut config) = self.config.try_write() {
                if ui.button("Request").clicked() {
                    (*config).active_window = ActiveWindow::REQUEST;
                }
                if ui.button("Environment").clicked() {
                    (*config).active_window = ActiveWindow::ENVIRONMENT;
                }
                if ui.button("History").clicked() {
                    (*config).active_window = ActiveWindow::HISTORY;
                }
            }
        });
        if let Ok(mut config) = self.config.try_write() {
            let active_window = &config.active_window;
            SidePanel::left("content_panel").show(ctx, |ui| match active_window {
                ActiveWindow::REQUEST => {
                    ui.label("Collections");
                }
                ActiveWindow::ENVIRONMENT => {
                    ui.label("Environments");
                }
                ActiveWindow::HISTORY => {
                    ui.label("History");
                }
            });
            TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.heading("Welcome to Postie!");
                ui.horizontal(|ui| {
                    let mut method = &config.selected_http_method;
                    ComboBox::from_label("")
                        .selected_text(format!("{:?}", method))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut method, &HttpMethod::GET, "GET");
                            ui.selectable_value(&mut method, &HttpMethod::POST, "POST");
                            ui.selectable_value(&mut method, &HttpMethod::PUT, "PUT");
                            ui.selectable_value(&mut method, &HttpMethod::DELETE, "DELETE");
                            ui.selectable_value(&mut method, &HttpMethod::PATCH, "PATCH");
                            ui.selectable_value(&mut method, &HttpMethod::OPTIONS, "OPTIONS");
                            ui.selectable_value(&mut method, &HttpMethod::HEAD, "HEAD");
                        });
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut config.url);
                    if ui.button("Submit").clicked() {
                        let body = if config.selected_http_method != HttpMethod::GET {
                            Some(
                                serde_json::from_str(&config.body_str)
                                    .expect("Body is invalid json"),
                            )
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
                            method: config.selected_http_method.clone(),
                            url: config.url.clone(),
                            environment: self.environment.borrow().clone(),
                        };

                        let _ = Gui::spawn_submit(self, request);
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Environment").clicked() {
                        (*config).request_window_mode = RequestWindowMode::ENVIRONMENT;
                    }
                    if ui.button("Params").clicked() {
                        (*config).request_window_mode = RequestWindowMode::PARAMS;
                    }
                    if ui.button("Headers").clicked() {
                        (*config).request_window_mode = RequestWindowMode::HEADERS;
                    }
                    if ui.button("Body").clicked() {
                        (*config).request_window_mode = RequestWindowMode::BODY;
                    }
                });
            });
            match config.request_window_mode {
                RequestWindowMode::BODY => {
                    TopBottomPanel::top("request_panel")
                        .resizable(true)
                        .min_height(250.0)
                        .show(ctx, |ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    TextEdit::multiline(&mut config.body_str)
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
                                    ui.strong("Type");
                                });
                                header.col(|ui| {
                                    ui.strong("Value");
                                });
                            })
                            .body(|mut body| {
                                if let Some(environemnt) = self.environment.borrow_mut().clone() {
                                    if let Some(values) = environemnt.values {
                                        for mut value in values {
                                            body.row(30.0, |mut row| {
                                                row.col(|ui| {
                                                    ui.checkbox(&mut value.enabled, "");
                                                });
                                                row.col(|ui| {
                                                    ui.text_edit_singleline(&mut value.key);
                                                });
                                                row.col(|ui| {
                                                    ui.text_edit_singleline(&mut value.r#type);
                                                });
                                                row.col(|ui| {
                                                    ui.text_edit_singleline(&mut value.value);
                                                });
                                            });
                                        }
                                    }
                                }
                                body.row(30.0, |mut row| {
                                    row.col(|ui| {
                                        if ui.button("Add").clicked() {
                                            if let Some(environemnt) =
                                                self.environment.borrow_mut().clone()
                                            {
                                                if let Some(mut values) = environemnt.values {
                                                    values.push(EnvironmentValue {
                                                        key: String::from(""),
                                                        value: String::from(""),
                                                        r#type: String::from("default"),
                                                        enabled: true,
                                                    });
                                                }
                                            }
                                        };
                                    });
                                });
                            });
                    });
                }
            };
            if config.import_window_open == true {
                egui::Window::new("Import Modal")
                    .open(&mut config.import_window_open.clone())
                    .show(ctx, |ui| {
                        ui.label("Please copy and paste the file path to import");
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut config.import_file_path.clone());
                            if ui.button("Import").clicked() {
                                let path = config.import_file_path.to_owned();
                                let _ = match config.import_mode {
                                    ImportMode::COLLECTION => {
                                        tokio::spawn(async move {
                                            PostieApi::import_collection(&path).await
                                        });
                                    }
                                    ImportMode::ENVIRONMENT => {
                                        tokio::spawn(async move {
                                            PostieApi::import_environment(&path).await
                                        });
                                    }
                                };
                            };
                        });
                    });
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let _con = initialize_db().await;
    println!("{:?}", _con);
    let app = Gui::default();
    let native_options = NativeOptions::default();
    let _ = eframe::run_native("Postie", native_options, Box::new(|_cc| Box::new(app)));
}
