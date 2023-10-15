use api::{HttpMethod, HttpRequest, PostieApi};
use eframe::{
    egui::{CentralPanel, ComboBox, ScrollArea, SidePanel, TextEdit, TopBottomPanel},
    App, NativeOptions,
};
use egui::TextStyle;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cell::RefCell, collections::HashSet, error::Error, rc::Rc};
use tokio::runtime;
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
    ENVIRONMENT
}
#[derive(Serialize, Deserialize)]
pub enum RequestWindowMode {
    PARAMS,
    HEADERS,
    BODY,
}

#[derive(Serialize, Deserialize)]
pub struct GuiConfig {
    pub active_window: ActiveWindow,
    pub request_window_mode: RequestWindowMode,
    pub selected_http_method: HttpMethod,
    pub url: String,
    pub body_str: String,
    pub headers: Rc<RefCell<Vec<(bool, String, String)>>>,
    pub response: Option<Value>,
    pub import_window_open: bool,
    pub import_mode: ImportMode,
    pub import_file_path: String
}
impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            active_window: ActiveWindow::REQUEST,
            request_window_mode: RequestWindowMode::BODY,
            selected_http_method: HttpMethod::GET,
            url: String::from("https://httpbin.org/json"),
            body_str: String::from("{ \"foo\": \"bar\" }"),
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
            response: None,
            import_window_open: false,
            import_file_path: String::from(""),
            import_mode: ImportMode::COLLECTION
        }
    }
}

pub struct Gui {
    pub config: GuiConfig,
    pub rt: runtime::Runtime,
}
impl Default for Gui {
    fn default() -> Self {
        Self {
            config: GuiConfig::default(),
            rt: runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }
}
impl Gui {
    fn spawn_submit(&mut self, input: HttpRequest) -> Result<Value, Box<dyn Error>> {
        let result = self.rt.block_on(async {
            let api_response = PostieApi::make_request(input).await;
            api_response
        });
        Ok(result.unwrap())
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                            self.config.import_window_open = true;
                            self.config.import_mode = ImportMode::COLLECTION;
                        };
                        if ui.button("Environment").clicked() {};
                    });
                    ui.menu_button("Export", |ui| {
                        if ui.button("Collection").clicked() {
                            self.config.import_window_open = true;
                            self.config.import_mode = ImportMode::ENVIRONMENT;
                        };
                        if ui.button("Environment").clicked() {};
                    });
                });
            });
        });
        SidePanel::left("nav_panel").show(ctx, |ui| {
            if ui.button("Request").clicked() {
                self.config.active_window = ActiveWindow::REQUEST;
            }
            if ui.button("Environment").clicked() {
                self.config.active_window = ActiveWindow::ENVIRONMENT;
            }
            if ui.button("History").clicked() {
                self.config.active_window = ActiveWindow::HISTORY;
            }
        });
        SidePanel::left("content_panel").show(ctx, |ui| match self.config.active_window {
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
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.config.selected_http_method))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::GET,
                            "GET",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::POST,
                            "POST",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::PUT,
                            "PUT",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::DELETE,
                            "DELETE",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::PATCH,
                            "PATCH",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::OPTIONS,
                            "OPTIONS",
                        );
                        ui.selectable_value(
                            &mut self.config.selected_http_method,
                            HttpMethod::HEAD,
                            "HEAD",
                        );
                    });
                ui.label("URL:");
                ui.text_edit_singleline(&mut self.config.url);
                if ui.button("Submit").clicked() {
                    let body = if self.config.selected_http_method != HttpMethod::GET {
                        Some(
                            serde_json::from_str(&self.config.body_str)
                                .expect("Body is invalid json"),
                        )
                    } else {
                        None
                    };
                    let submitted_headers = self
                        .config
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
                        method: self.config.selected_http_method.clone(),
                        url: self.config.url.clone(),
                    };

                    let response = Gui::spawn_submit(self, request);
                    if response.is_ok() {
                        self.config.response = Some(response.unwrap());
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Params").clicked() {
                    self.config.request_window_mode = RequestWindowMode::PARAMS;
                }
                if ui.button("Headers").clicked() {
                    self.config.request_window_mode = RequestWindowMode::HEADERS;
                }
                if ui.button("Body").clicked() {
                    self.config.request_window_mode = RequestWindowMode::BODY;
                }
            });
        });
        match self.config.request_window_mode {
            RequestWindowMode::BODY => {
                TopBottomPanel::top("request_panel")
                    .resizable(true)
                    .min_height(250.0)
                    .show(ctx, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut self.config.body_str)
                                    .code_editor()
                                    .desired_rows(20)
                                    .lock_focus(true)
                                    .font(TextStyle::Monospace),
                            );
                        });
                    });
                if self.config.response.is_some() {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.label(self.config.response.as_ref().unwrap().to_string());
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
                            for header in self.config.headers.borrow_mut().iter_mut() {
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
                                        self.config.headers.borrow_mut().push((
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
        };
        if self.config.import_window_open == true {
            egui::Window::new("Import Modal")
                .open(&mut self.config.import_window_open)
                .show(ctx, |ui| {
                    ui.label("Please copy and paste the file path to import");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.config.import_file_path);
                        if ui.button("Import").clicked() {
                            self.rt.block_on(async {
                                let path = self.config.import_file_path.to_owned();
                                let _ = match self.config.import_mode {
                                    ImportMode::COLLECTION => PostieApi::import_collection(&path).await,
                                    ImportMode::ENVIRONMENT => todo!(),
                                };
                            });
                        };
                    });
                });
        }
    }
}

fn main() {
    let app = Gui::default();
    let native_options = NativeOptions::default();
    let _ = eframe::run_native("Postie", native_options, Box::new(|_cc| Box::new(app)));
}
