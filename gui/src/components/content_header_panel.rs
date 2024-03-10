use api::{HttpMethod, HttpRequest};
use egui::{ComboBox, TopBottomPanel};
use uuid::Uuid;

use crate::{AuthMode, Gui, RequestWindowMode};

pub fn content_header_panel(gui: &mut Gui, ctx: &egui::Context) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("Welcome to Postie!");
        ui.horizontal(|ui| {
            ComboBox::from_label("")
                .selected_text(format!("{:?}", gui.selected_http_method))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut gui.selected_http_method, HttpMethod::GET, "GET");
                    ui.selectable_value(&mut gui.selected_http_method, HttpMethod::POST, "POST");
                    ui.selectable_value(&mut gui.selected_http_method, HttpMethod::PUT, "PUT");
                    ui.selectable_value(
                        &mut gui.selected_http_method,
                        HttpMethod::DELETE,
                        "DELETE",
                    );
                    ui.selectable_value(&mut gui.selected_http_method, HttpMethod::PATCH, "PATCH");
                    ui.selectable_value(
                        &mut gui.selected_http_method,
                        HttpMethod::OPTIONS,
                        "OPTIONS",
                    );
                    ui.selectable_value(&mut gui.selected_http_method, HttpMethod::HEAD, "HEAD");
                });
            ui.label("URL:");
            ui.text_edit_singleline(&mut gui.url);
            if ui.button("Submit").clicked() {
                let body = if gui.selected_http_method != HttpMethod::GET {
                    Some(serde_json::from_str(&gui.body_str).expect("Body is invalid json"))
                } else {
                    None
                };
                let mut submitted_headers: Vec<(String, String)> = gui
                    .headers
                    .borrow_mut()
                    .iter()
                    .filter(|h| h.0 == true)
                    .map(|h| (h.1.to_owned(), h.2.to_owned()))
                    .collect();
                match gui.selected_auth_mode {
                    AuthMode::APIKEY => {
                        submitted_headers.push(
                            (String::from("x-api-key"), gui.api_key.clone())
                        );
                    },
                    AuthMode::BEARER => {
                        submitted_headers.push(
                            (String::from("Authorization"),
                            format!("Bearer {}", gui.api_key))
                        );
                    },
                    AuthMode::NONE => (),
                };
                let request = HttpRequest {
                    id: Uuid::new_v4(),
                    name: None,
                    headers: Some(Gui::remove_duplicate_headers(submitted_headers)),
                    body,
                    method: gui.selected_http_method.clone(),
                    url: gui.url.clone(),
                    environment: gui.selected_environment.borrow().clone(),
                };

                let _ = Gui::spawn_submit(gui, request);
            }
        });
        if let Ok(mut request_window_mode) = gui.request_window_mode.try_write() {
            ui.horizontal(|ui| {
                if ui.button("Environment").clicked() {
                    *request_window_mode = RequestWindowMode::ENVIRONMENT;
                }
                if ui.button("Params").clicked() {
                    *request_window_mode = RequestWindowMode::PARAMS;
                }
                if ui.button("Auth").clicked() {
                    *request_window_mode = RequestWindowMode::AUTHORIZATION;
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
}
