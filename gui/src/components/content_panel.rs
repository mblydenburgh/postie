use std::cell::RefMut;

use api::{domain::environment::EnvironmentValue, ResponseData};
use egui::{CentralPanel, ComboBox, ScrollArea, TextEdit, TextStyle, TopBottomPanel};
use egui_extras::{Column, TableBuilder};
use egui_json_tree::JsonTree;

use crate::{AuthMode, Gui, RequestWindowMode};

pub fn content_panel(gui: &mut Gui, ctx: &egui::Context) {
    let sender = &mut gui.sender.clone();
    if let Ok(request_window_mode) = gui.request_window_mode.try_read() {
        match *request_window_mode {
            RequestWindowMode::BODY => {
                TopBottomPanel::top("request_panel")
                    .resizable(true)
                    .min_height(250.0)
                    .show(ctx, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut gui.body_str)
                                    .code_editor()
                                    .desired_rows(20)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY)
                                    .font(TextStyle::Monospace),
                            );
                        });
                    });
                if gui.response.try_read().unwrap().is_some() {
                    CentralPanel::default().show(ctx, |ui| {
                        let binding = gui.response.try_read().unwrap();
                        let r = binding.as_ref().unwrap();
                        match r {
                            ResponseData::JSON(json) => {
                                ScrollArea::vertical().show(ui, |ui| {
                                    JsonTree::new("response-json", json).show(ui);
                                });
                            }
                            ResponseData::TEXT(text) => {
                                ScrollArea::vertical().show(ui, |ui| {
                                    ui.label(text);
                                });
                            }
                        };
                    });
                }
            }
            RequestWindowMode::PARAMS => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.label("params");
                });
            }
            RequestWindowMode::AUTHORIZATION => {
                CentralPanel::default().show(ctx, |ui| {
                    ComboBox::from_label("")
                        .selected_text(format!("{:?}", gui.selected_auth_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut gui.selected_auth_mode,
                                AuthMode::BEARER,
                                "Bearer",
                            );
                            ui.selectable_value(
                                &mut gui.selected_auth_mode,
                                AuthMode::APIKEY,
                                "Api Key",
                            );
                            ui.selectable_value(
                                &mut gui.selected_auth_mode,
                                AuthMode::OAUTH2,
                                "OAuth2",
                            );
                            ui.selectable_value(
                                &mut gui.selected_auth_mode,
                                AuthMode::NONE,
                                "None",
                            );
                        });
                    match gui.selected_auth_mode {
                        AuthMode::APIKEY => {
                            ui.label("Api Key Value");
                            ui.text_edit_multiline(&mut gui.api_key);
                            ui.label("Header Name");
                            ui.text_edit_singleline(&mut gui.api_key_name);
                        }
                        AuthMode::BEARER => {
                            ui.label("Enter Bearer Token");
                            ui.text_edit_multiline(&mut gui.bearer_token);
                        }
                        AuthMode::OAUTH2 => {
                            CentralPanel::default().show(ctx, |_ui| {
                                TopBottomPanel::top("oauth_request_panel")
                                    .resizable(true)
                                    .show(ctx, |ui| {
                                        ui.heading("Configure New Token");
                                        ui.horizontal(|ui| {
                                            ui.label("Access Token Url");
                                            ui.text_edit_singleline(
                                                &mut gui.oauth_config.access_token_url,
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Client ID");
                                            ui.text_edit_singleline(
                                                &mut gui.oauth_config.client_id,
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Client Secret");
                                            ui.text_edit_singleline(
                                                &mut gui.oauth_config.client_secret,
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Scope");
                                            ui.text_edit_singleline(
                                                &mut gui.oauth_config.request.scope,
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Audience");
                                            ui.text_edit_singleline(
                                                &mut gui.oauth_config.request.audience,
                                            );
                                        });

                                        if ui.button("Request Token").clicked() {
                                            println!("requesting token");
                                            let oauth_input = api::OAuth2Request {
                                                access_token_url: gui
                                                    .oauth_config
                                                    .access_token_url
                                                    .clone(),
                                                refresh_url: gui.oauth_config.refresh_url.clone(),
                                                client_id: gui.oauth_config.client_id.clone(),
                                                client_secret: gui
                                                    .oauth_config
                                                    .client_secret
                                                    .clone(),
                                                request: api::OAuthRequestBody {
                                                    grant_type: gui
                                                        .oauth_config
                                                        .request
                                                        .grant_type
                                                        .clone(),
                                                    scope: gui.oauth_config.request.scope.clone(),
                                                    audience: gui
                                                        .oauth_config
                                                        .request
                                                        .audience
                                                        .clone(),
                                                },
                                            };
                                            let _ = Gui::spawn_ouath_request(sender, oauth_input);
                                        };
                                    });
                                if let Ok(token_read_guard) = gui.oauth_response.try_read() {
                                    if let Some(response_data) = &*token_read_guard {
                                        TopBottomPanel::bottom("ouath_response")
                                            .resizable(true)
                                            .show(ctx, |ui| {
                                                match response_data {
                                                    ResponseData::JSON(j) => {
                                                        let token = serde_json::from_value::<
                                                            api::OAuthResponse,
                                                        >(
                                                            j.clone()
                                                        )
                                                        .unwrap();
                                                        ui.label(format!(
                                                            "Retrieved token: {}",
                                                            token.access_token
                                                        ));
                                                    }
                                                    ResponseData::TEXT(t) => todo!(),
                                                };
                                            });
                                    }
                                }
                            });
                        }
                        AuthMode::NONE => (),
                    };
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
                            for header in gui.headers.borrow_mut().iter_mut() {
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
                                        gui.headers.borrow_mut().push((
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
                            let selected_environment = gui.selected_environment.borrow_mut();
                            let mut values_ref =
                                RefMut::map(selected_environment, |env| &mut env.values);
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
}
