use std::{
  cell::RefCell,
  rc::Rc,
  sync::{Arc, Mutex},
};

use api::domain::{
  environment::{EnvironmentFile, EnvironmentValue},
  request,
  response::{OAuthResponse, ResponseData},
  ui::{self},
};
use egui::{CentralPanel, ScrollArea, TextEdit, TextStyle, TopBottomPanel};
use egui_extras::{Column, TableBuilder};
use egui_json_tree::JsonTree;
use tokio::sync::RwLock;

use crate::{events, GuiState, ThreadSafeState};

pub struct ContentPanel {}

impl ContentPanel {
  pub fn new() -> Self {
    Self {}
  }

  pub fn show(
    &mut self,
    ctx: &egui::Context,
    gui_state: &GuiState,
    worker_state: &ThreadSafeState,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    let mode = if let Ok(mode_guard) = gui_state.request_window_mode.try_read() {
      *mode_guard
    } else {
      ui::RequestWindowMode::BODY
    };

    match mode {
      ui::RequestWindowMode::BODY => {
        self.render_body_tab(ctx, &mut gui_state.body_str.clone(), &worker_state.response)
      }
      ui::RequestWindowMode::AUTHORIZATION => {
        self.render_auth_tab(
          ctx,
          &mut gui_state.selected_auth_mode.clone(),
          &mut gui_state.api_key_name.clone(),
          &mut gui_state.api_key.clone(),
          &mut gui_state.bearer_token.clone(),
          &mut gui_state.oauth_config.clone(),
          &mut gui_state.oauth_token.clone(),
          &worker_state.received_token,
          &worker_state.oauth_response,
          event_tx,
        );
      }
      ui::RequestWindowMode::HEADERS => self.render_headers_tab(ctx, &gui_state.headers),
      ui::RequestWindowMode::ENVIRONMENT => {
        self.render_environment_tab(ctx, &gui_state.selected_environment)
      }
      ui::RequestWindowMode::PARAMS => {
        self.render_params_tab(ctx, &mut gui_state.url.clone());
      }
    }
  }

  fn render_body_tab(
    &mut self,
    ctx: &egui::Context,
    body_str: &mut String,
    response_lock: &Arc<RwLock<Option<ResponseData>>>,
  ) {
    TopBottomPanel::top("request_panel")
      .resizable(true)
      .min_height(250.0)
      .show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
          ui.add(
            TextEdit::multiline(body_str)
              .code_editor()
              .desired_width(f32::INFINITY)
              .font(TextStyle::Monospace),
          );
        });
      });

    CentralPanel::default().show(ctx, |ui| {
      if let Ok(res_guard) = response_lock.try_read() {
        if let Some(res) = res_guard.as_ref() {
          ScrollArea::vertical().show(ui, |ui| match res {
            ResponseData::JSON(json) => {
              JsonTree::new("res", json).show(ui);
            }
            ResponseData::TEXT(t) | ResponseData::XML(t) | ResponseData::UNKNOWN(t) => {
              ui.label(t);
            }
          });
        }
      }
    });
  }

  fn render_auth_tab(
    &mut self,
    ctx: &egui::Context,
    auth_mode: &mut ui::AuthMode,
    api_key_name: &mut String,
    api_key_value: &mut String,
    bearer_token: &mut String,
    oauth_config: &mut request::OAuth2Request,
    oauth_token: &mut String,
    received_token_lock: &Arc<Mutex<bool>>,
    oauth_response_lock: &Arc<RwLock<Option<ResponseData>>>,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    CentralPanel::default().show(ctx, |ui| {
      ui.heading("Authentication");

      ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("auth_type_selector")
          .selected_text(format!("{:?}", auth_mode))
          .show_ui(ui, |ui| {
            ui.selectable_value(auth_mode, ui::AuthMode::NONE, "No Auth");
            ui.selectable_value(auth_mode, ui::AuthMode::APIKEY, "API Key");
            ui.selectable_value(auth_mode, ui::AuthMode::BEARER, "Bearer Token");
            ui.selectable_value(auth_mode, ui::AuthMode::OAUTH2, "OAuth 2.0");
          });
      });

      ui.separator();

      match auth_mode {
        ui::AuthMode::APIKEY => {
          egui::Grid::new("api_key_leaf_grid")
            .num_columns(2)
            .show(ui, |ui| {
              ui.label("Header Name:");
              ui.text_edit_singleline(api_key_name);
              ui.end_row();
              ui.label("Value:");
              ui.text_edit_singleline(api_key_value);
              ui.end_row();
            });
        }
        ui::AuthMode::BEARER => {
          ui.label("Bearer Token:");
          ui.add(
            egui::TextEdit::multiline(bearer_token)
              .hint_text("eyJhbGciOiJIUzI1...")
              .desired_rows(8)
              .font(egui::TextStyle::Monospace)
              .desired_width(f32::INFINITY),
          );
        }
        ui::AuthMode::OAUTH2 => {
          // Sub-leaf for the OAuth configuration grid
          self.render_oauth_details(ui, oauth_config, oauth_token, event_tx);

          // Logic to ingest the token from the background worker
          if let Ok(mut lock) = received_token_lock.lock() {
            if !*lock {
              if let Ok(res_guard) = oauth_response_lock.try_read() {
                if let Some(ResponseData::JSON(j)) = res_guard.as_ref() {
                  if let Ok(data) = serde_json::from_value::<OAuthResponse>(j.clone()) {
                    *oauth_token = data.access_token;
                    *lock = true;
                  }
                }
              }
            }
          }
        }
        ui::AuthMode::NONE => {
          ui.weak("No authentication headers will be sent with this request.");
        }
      }
    });
  }

  fn render_oauth_details(
    &mut self,
    ui: &mut egui::Ui,
    config: &mut request::OAuth2Request,
    current_token: &String,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    egui::Grid::new("oauth_details")
      .num_columns(2)
      .spacing([20.0, 8.0])
      .show(ui, |ui| {
        ui.label("Access Token URL:");
        ui.text_edit_singleline(&mut config.access_token_url);
        ui.end_row();

        ui.label("Client ID:");
        ui.text_edit_singleline(&mut config.client_id);
        ui.end_row();

        ui.label("Client Secret:");
        ui.add(egui::TextEdit::singleline(&mut config.client_secret).password(true));
        ui.end_row();
      });

    if ui.button("Get New Access Token").clicked() {
      let _ = event_tx.try_send(events::GuiEvent::SubmitOAuth2Request(config.clone()));
    }

    if !current_token.is_empty() {
      ui.add_space(10.0);
      ui.label("Current Token:");
      ui.add(egui::Label::new(current_token).wrap().selectable(true));
    }
  }

  fn render_headers_tab(
    &mut self,
    ctx: &egui::Context,
    headers_rc: &Rc<RefCell<Vec<(bool, String, String)>>>,
  ) {
    CentralPanel::default().show(ctx, |ui| {
      let mut _add_clicked = false;
      TableBuilder::new(ui)
        .column(Column::auto())
        .column(Column::remainder())
        .column(Column::remainder())
        .header(20.0, |mut h| {
          h.col(|ui| {
            ui.label("On");
          });
          h.col(|ui| {
            ui.label("Key");
          });
          h.col(|ui| {
            ui.label("Value");
          });
        })
        .body(|mut body| {
          let mut headers = headers_rc.borrow_mut();
          for (enabled, key, value) in headers.iter_mut() {
            body.row(25.0, |mut row| {
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
        });

      if ui.button("Add Header").clicked() {
        headers_rc.borrow_mut().push((true, "".into(), "".into()));
      }
    });
  }

  fn render_environment_tab(&mut self, ctx: &egui::Context, env_rc: &Rc<RefCell<EnvironmentFile>>) {
    CentralPanel::default().show(ctx, |ui| {
      ui.heading("Environment Variables");

      let mut add_clicked = false;

      egui::ScrollArea::vertical().show(ui, |ui| {
        TableBuilder::new(ui)
          .striped(true)
          .column(Column::auto()) // Enabled
          .column(Column::remainder()) // Key
          .column(Column::remainder()) // Value
          .column(Column::auto()) // Type
          .header(20.0, |mut header| {
            header.col(|ui| {
              ui.strong("On");
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
            let mut env = env_rc.borrow_mut();
            if let Some(values) = env.values.as_mut() {
              for var in values {
                body.row(25.0, |mut row| {
                  row.col(|ui| {
                    ui.checkbox(&mut var.enabled, "");
                  });
                  row.col(|ui| {
                    ui.text_edit_singleline(&mut var.key);
                  });
                  row.col(|ui| {
                    ui.text_edit_singleline(&mut var.value);
                  });
                  row.col(|ui| {
                    ui.text_edit_singleline(&mut var.r#type);
                  });
                });
              }
            }
          });
      });

      if ui.button("Add Variable").clicked() {
        add_clicked = true;
      }

      if add_clicked {
        let mut env = env_rc.borrow_mut();
        let new_var = EnvironmentValue {
          key: "".into(),
          value: "".into(),
          r#type: "default".into(),
          enabled: true,
        };
        if let Some(values) = env.values.as_mut() {
          values.push(new_var);
        } else {
          env.values = Some(vec![new_var]);
        }
      }
    });
  }
  fn render_params_tab(&mut self, ctx: &egui::Context, url: &mut String) {
    CentralPanel::default().show(ctx, |ui| {
      ui.heading("Query Parameters");

      let parts: Vec<&str> = url.splitn(2, '?').collect();
      let base_url = parts[0].to_string();
      let mut params: Vec<(String, String)> = parts
        .get(1)
        .map(|p| {
          p.split('&')
            .filter(|s| !s.is_empty())
            .map(|pair| {
              let kv: Vec<&str> = pair.splitn(2, '=').collect();
              (kv[0].to_string(), kv.get(1).unwrap_or(&"").to_string())
            })
            .collect()
        })
        .unwrap_or_default();

      let mut changed = false;

      TableBuilder::new(ui)
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::auto())
        .header(20.0, |mut h| {
          h.col(|ui| {
            ui.strong("Key");
          });
          h.col(|ui| {
            ui.strong("Value");
          });
          h.col(|ui| {
            ui.label("");
          });
        })
        .body(|mut body| {
          for i in 0..params.len() {
            body.row(25.0, |mut row| {
              row.col(|ui| {
                if ui.text_edit_singleline(&mut params[i].0).changed() {
                  changed = true;
                }
              });
              row.col(|ui| {
                if ui.text_edit_singleline(&mut params[i].1).changed() {
                  changed = true;
                }
              });
              row.col(|ui| {
                if ui.button("🗑").clicked() {
                  params.remove(i);
                  changed = true;
                }
              });
            });
          }
        });

      if ui.button("Add Param").clicked() {
        params.push(("".into(), "".into()));
        changed = true;
      }

      // Reconstruct the URL if the table changed
      if changed {
        if params.is_empty() {
          *url = base_url;
        } else {
          let query = params
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
          *url = format!("{}?{}", base_url, query);
        }
      }
    });
  }
}
