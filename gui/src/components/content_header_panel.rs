use std::borrow::BorrowMut;

use api::domain::{request, ui};
use egui::{ComboBox, TopBottomPanel};
use uuid::Uuid;

use crate::Gui;

pub fn content_header_panel(gui: &mut Gui, ctx: &egui::Context) {
  TopBottomPanel::top("top_panel").show(ctx, |ui| {
    ui.horizontal(|ui| {
      ComboBox::from_label("")
        .selected_text(format!("{:?}", gui.selected_http_method))
        .show_ui(ui, |ui| {
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::GET,
            "GET",
          );
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::POST,
            "POST",
          );
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::PUT,
            "PUT",
          );
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::DELETE,
            "DELETE",
          );
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::PATCH,
            "PATCH",
          );
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::OPTIONS,
            "OPTIONS",
          );
          ui.selectable_value(
            &mut gui.selected_http_method,
            request::HttpMethod::HEAD,
            "HEAD",
          );
        });
      ui.label("URL:");
      ui.add(egui::TextEdit::singleline(&mut gui.url).desired_width(400.0));
      if ui.button("Submit").clicked() {
        let body = if gui.selected_http_method != request::HttpMethod::GET {
          Some(request::RequestBody::JSON(
            serde_json::from_str(&gui.body_str).unwrap_or_default(),
          ))
        } else {
          None
        };
        // take headers from gui.headers and convert to Vec<(String, String)>
        let mut submitted_headers: Vec<(String, String)> = (*gui
          .headers
          .borrow()
          .iter()
          .filter(|h| h.0)
          .map(|h| (h.1.to_owned(), h.2.to_owned()))
          .collect::<Vec<(String, String)>>())
        .to_vec();
        match gui.selected_auth_mode {
          ui::AuthMode::APIKEY => {
            submitted_headers.push((gui.api_key_name.clone(), gui.api_key.clone()));
          }
          ui::AuthMode::BEARER => {
            submitted_headers.push((
              String::from("Authorization"),
              format!("Bearer {}", gui.bearer_token),
            ));
          }
          ui::AuthMode::OAUTH2 => {
            submitted_headers.push((
              String::from("Authorization"),
              format!("Bearer {}", gui.oauth_token),
            ));
          }
          ui::AuthMode::NONE => (),
        };

        let active_tab_guard = gui.active_tab.borrow_mut();
        let tab_id = if let Some(active_tab) = active_tab_guard.try_read().unwrap().as_ref() {
          Uuid::parse_str(&active_tab.id).unwrap()
        } else {
          Uuid::new_v4()
        };
        let request = request::HttpRequest {
          tab_id,
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
          *request_window_mode = ui::RequestWindowMode::ENVIRONMENT;
        }
        /*if ui.button("Params").clicked() {
            *request_window_mode = RequestWindowMode::PARAMS;
        }*/
        if ui.button("Auth").clicked() {
          *request_window_mode = ui::RequestWindowMode::AUTHORIZATION;
        }
        if ui.button("Headers").clicked() {
          *request_window_mode = ui::RequestWindowMode::HEADERS;
        }
        if ui.button("Body").clicked() {
          *request_window_mode = ui::RequestWindowMode::BODY;
        }
      });
    }
  });
}
