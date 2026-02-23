use std::{cell::RefCell, collections::HashSet, rc::Rc, sync::Arc};

use api::domain::{
  environment::EnvironmentFile,
  request::{self, HttpMethod, HttpRequest},
  tab::Tab,
  ui::{self, RequestWindowMode},
};
use egui::{ComboBox, InnerResponse, TopBottomPanel};
use tokio::sync::{mpsc::Sender, RwLock};
use uuid::Uuid;

use crate::events::{self, GuiEvent};

pub struct ContentHeaderPanel {
  pub selected_http_method: HttpMethod,
  pub request_window_mode: Arc<RwLock<RequestWindowMode>>,
}

impl ContentHeaderPanel {
  pub fn new() -> Self {
    Self {
      selected_http_method: HttpMethod::GET,
      request_window_mode: Arc::new(RwLock::new(RequestWindowMode::BODY)),
    }
  }
  pub fn show(
    &mut self,
    ctx: &egui::Context,
    event_tx: &Sender<GuiEvent>,
    active_tab: Arc<RwLock<Tab>>,
    environment: Rc<RefCell<EnvironmentFile>>,
    headers: Rc<RefCell<Vec<(bool, String, String)>>>,
    auth_mode: ui::AuthMode,
    api_key_name: String,
    api_key: String,
    bearer_token: String,
    oauth_token: String,
  ) -> InnerResponse<Option<HttpRequest>> {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
      self.render_url_bar(
        ui,
        event_tx,
        active_tab,
        environment,
        headers,
        auth_mode,
        api_key_name,
        api_key,
        bearer_token,
        oauth_token,
      )
    })
  }

  fn render_url_bar(
    &mut self,
    ui: &mut egui::Ui,
    event_tx: &Sender<GuiEvent>,
    active_tab: Arc<RwLock<Tab>>,
    environment: Rc<RefCell<EnvironmentFile>>,
    headers: Rc<RefCell<Vec<(bool, String, String)>>>,
    auth_mode: ui::AuthMode,
    api_key_name: String,
    api_key: String,
    bearer_token: String,
    oauth_token: String,
  ) -> Option<HttpRequest> {
    ui.horizontal(|ui| {
      // HTTP Method Selector
      self.render_method_selector(ui);

      if let Ok(mut tab) = active_tab.try_write() {
        // URL Input
        ui.label("URL:");
        ui.add(egui::TextEdit::singleline(&mut tab.url).desired_width(400.0));
      }

      // Submit Button
      if ui.button("Submit").clicked() {
        if let Some(req) = self.build_request(
          active_tab,
          environment,
          headers,
          auth_mode,
          api_key_name,
          api_key,
          bearer_token,
          oauth_token,
        ) {
          event_tx
            .try_send(events::GuiEvent::SubmitRequest(req))
            .unwrap();
        };
      }
    });

    self.render_mode_switcher(ui);
    None
  }

  fn render_method_selector(&mut self, ui: &mut egui::Ui) {
    ComboBox::from_label("")
      .selected_text(format!("{:?}", self.selected_http_method))
      .show_ui(ui, |ui| {
        // List all variants explicitly to avoid ownership issues
        let variants = [
          (HttpMethod::GET, "GET"),
          (HttpMethod::POST, "POST"),
          (HttpMethod::PUT, "PUT"),
          (HttpMethod::DELETE, "DELETE"),
          (HttpMethod::PATCH, "PATCH"),
          (HttpMethod::OPTIONS, "OPTIONS"),
          (HttpMethod::HEAD, "HEAD"),
        ];

        for (method, label) in variants {
          ui.selectable_value(&mut self.selected_http_method, method, label);
        }
      });
  }
  fn render_mode_switcher(&mut self, ui: &mut egui::Ui) {
    if let Ok(mut mode) = self.request_window_mode.try_write() {
      ui.horizontal(|ui| {
        for (label, target_mode) in [
          ("Environment", ui::RequestWindowMode::ENVIRONMENT),
          ("Auth", ui::RequestWindowMode::AUTHORIZATION),
          ("Headers", ui::RequestWindowMode::HEADERS),
          ("Body", ui::RequestWindowMode::BODY),
        ] {
          if ui.button(label).clicked() {
            *mode = target_mode;
          }
        }
      });
    }
  }

  fn build_request(
    &self,
    active_tab: Arc<RwLock<Tab>>,
    environment: Rc<RefCell<EnvironmentFile>>,
    headers: Rc<RefCell<Vec<(bool, String, String)>>>,
    auth_mode: ui::AuthMode,
    api_key_name: String,
    api_key: String,
    bearer_token: String,
    oauth_token: String,
  ) -> Option<request::HttpRequest> {
    let body = if active_tab.try_read().unwrap().method != request::HttpMethod::GET {
      Some(request::RequestBody::JSON(
        serde_json::from_str(&active_tab.try_read().unwrap().req_body).unwrap_or_default(),
      ))
    } else {
      None
    };
    let active_tab_guard = Arc::clone(&active_tab);
    let processed_headers = self.process_headers(
      &*headers.borrow(),
      auth_mode,
      api_key_name,
      api_key,
      bearer_token,
      oauth_token,
    );

    match active_tab_guard.clone().try_read() {
      Ok(tab) => Some(request::HttpRequest {
        tab_id: tab.id,
        id: Uuid::new_v4(),
        name: None,
        headers: Some(processed_headers),
        body,
        method: tab.method.clone(),
        url: tab.url.clone(),
        environment: environment.borrow().clone(),
      }),
      Err(_) => None,
    }
  }

  fn process_headers(
    &self,
    headers: &Vec<(bool, String, String)>,
    auth_mode: ui::AuthMode,
    api_key_name: String,
    api_key: String,
    bearer_token: String,
    oauth_token: String,
  ) -> Vec<(String, String)> {
    let mut headers = headers
      .iter()
      .filter(|h| h.0)
      .map(|h| (h.1.clone(), h.2.clone()))
      .collect::<Vec<_>>();

    match auth_mode {
      ui::AuthMode::APIKEY => headers.push((api_key_name, api_key)),
      ui::AuthMode::BEARER => {
        headers.push(("Authorization".into(), format!("Bearer {}", bearer_token)))
      }
      ui::AuthMode::OAUTH2 => {
        headers.push(("Authorization".into(), format!("Bearer {}", oauth_token)))
      }
      _ => (),
    }

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
