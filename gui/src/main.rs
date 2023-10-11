use api::{submit_request, HttpMethod, HttpRequest};
use eframe::{
    egui::{CentralPanel, ComboBox, SidePanel},
    epi::App,
    run_native, NativeOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use tokio::runtime;

#[derive(Serialize, Deserialize)]
pub enum ActiveWindow {
    REQUEST,
    ENVIRONMENT,
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
    pub response: Option<Value>,
}
impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            active_window: ActiveWindow::REQUEST,
            request_window_mode: RequestWindowMode::BODY,
            selected_http_method: HttpMethod::GET,
            url: String::from("https://localhost:3000"),
            response: None,
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
            let api_response = submit_request(input).await;
            api_response
        });
        Ok(result.unwrap())
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        SidePanel::left("nav_panel").show(ctx, |ui| {
            if ui.button("Request").clicked() {
                self.config.active_window = ActiveWindow::REQUEST;
            }
            if ui.button("Environment").clicked() {
                self.config.active_window = ActiveWindow::ENVIRONMENT;
            }
        });
        SidePanel::left("content_panel").show(ctx, |ui| match self.config.active_window {
            ActiveWindow::REQUEST => {
                ui.label("Collections");
            }
            ActiveWindow::ENVIRONMENT => {
                ui.label("Environments");
            }
        });
        CentralPanel::default().show(ctx, |ui| {
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
                    let request = HttpRequest {
                        name: None,
                        headers: None,
                        body: None,
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
                if ui.button("Params").clicked() {}
                if ui.button("Headers").clicked() {}
                if ui.button("Body").clicked() {}
            });
        });
        match self.config.request_window_mode {
            RequestWindowMode::BODY => {
                if self.config.response.is_some() {
                   CentralPanel::default().show(ctx, |ui| {
                       ui.label(self.config.response.as_ref().unwrap().to_string());
                   });
                }
            }
            RequestWindowMode::PARAMS => {}
            RequestWindowMode::HEADERS => {}
        }
    }

    fn name(&self) -> &str {
        "Postie"
    }
}

fn main() {
    let app = Gui::default();
    let native_options = NativeOptions::default();
    run_native(Box::new(app), native_options)
}
