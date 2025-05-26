use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};

use crate::Gui;
use api::{
  domain::{
    collection::{Collection, CollectionFolder, CollectionItemOrFolder},
    request::{DBRequest, HttpMethod},
    response::ResponseData,
    ui,
  },
  PostieApi,
};
use egui::{CollapsingResponse, InnerResponse, ScrollArea, SidePanel};

pub fn content_side_panel(gui: &mut Gui, ctx: &egui::Context) {
  let collections = {
    let lock = gui.collections.try_write().unwrap();
    lock.clone()
  };
  let active_winow_guard = {
    let lock = gui.active_window.try_write().unwrap();
    lock.clone()
  };
  SidePanel::left("content_panel").show(ctx, |ui| match active_winow_guard {
    ui::ActiveWindow::COLLECTIONS => {
      ScrollArea::vertical().show(ui, |ui| {
        ui.label("Collections");
        if let Some(cols) = collections {
          for c in cols {
            render_collection(ui, gui, &c);
          }
        }
      });
    }
    ui::ActiveWindow::ENVIRONMENT => {
      ScrollArea::vertical().show(ui, |ui| {
        ui.label("Environments");
        let envs_clone = Arc::clone(&gui.environments);
        let envs = envs_clone.try_write().unwrap();
        if let Some(env_vec) = &*envs {
          for env in env_vec {
            ui.selectable_value(
              &mut gui.selected_environment,
              Rc::new(RefCell::from(env.clone())),
              env.name.to_string(),
            );
          }
        }
      });
    }
    ui::ActiveWindow::HISTORY => {
      ScrollArea::vertical().show(ui, |ui| {
        ui.label("History");
        let history_items_clone = Arc::clone(&gui.request_history_items);
        let history_items = history_items_clone.try_write().unwrap();
        let request_clone = gui.saved_requests.try_write().unwrap();
        if let Some(item_vec) = &*history_items {
          for item in item_vec {
            let history_reqs = request_clone.as_ref().unwrap();
            let id = &item.clone().request_id;
            let req_name = history_reqs
              .get(id)
              .unwrap_or(&DBRequest {
                id: id.clone(),
                method: "GET".into(),
                url: "n/a".into(),
                name: None,
                headers: vec![],
                body: None,
              })
              .url
              .clone();
            if ui
              .selectable_value(
                &mut gui.selected_history_item,
                Rc::new(RefCell::from(Some(item.clone()))),
                format!("{:?}", req_name), // TODO - create function to get name
              )
              .clicked()
            {
              // TODO - replace url, method, request body, response body
              let responses_clone = gui.saved_responses.try_write().unwrap();
              let requests = request_clone.as_ref().unwrap();
              let responses = responses_clone.as_ref().unwrap();
              let historical_request = requests.get(&item.request_id).unwrap();
              let historical_response = responses.get(&item.response_id).unwrap();
              gui.url = historical_request.url.clone();
              gui.selected_http_method = HttpMethod::from_str(&historical_request.method).unwrap();
              match &historical_request.body {
                Some(body_json) => {
                  gui.body_str = body_json.to_string();
                }
                None => gui.body_str = String::from(""),
              }
              let ui_response_clone = gui.response.clone();
              let mut ui_response_guard = ui_response_clone.try_write().unwrap();
              let response_body = &historical_response.body;
              match response_body {
                Some(body) => {
                  let json_val = serde_json::json!(&body);
                  println!("val: {}", json_val);
                  let parsed_body = match serde_json::from_str(body) {
                    Ok(b) => ResponseData::JSON(b),
                    Err(e) => {
                      println!("{}", e);
                      ResponseData::TEXT(body.clone())
                    }
                  };
                  *ui_response_guard = Some(parsed_body)
                }
                None => *ui_response_guard = None,
              }
            }
          }
        }
      });
    }
  });
}

fn render_collection(
  ui: &mut egui::Ui,
  app: &mut Gui,
  c: &Collection,
) -> InnerResponse<CollapsingResponse<()>> {
  ui.horizontal(|ui| {
    if ui.button("X").clicked() {
      let clicked_id = c.info.id.clone();
      // call to delete collection by id, refresh collections for ui
      let refresh_clone = app.collections.clone();
      tokio::spawn(async move {
        let _ = PostieApi::delete_collection(clicked_id).await;
        Gui::refresh_collections(refresh_clone).await;
      });
    }
    ui.collapsing(c.info.name.clone(), |ui| {
      for i in c.item.clone() {
        match i {
          CollectionItemOrFolder::Item(item) => {
            if ui
              .selectable_value(
                &mut app.selected_request.clone(),
                Rc::new(RefCell::from(Some(item.clone().request))),
                item.name.to_string(),
              )
              .clicked()
            {
              // TODO - emit channel event to update gui fields
              app.url = item.request.url.raw.clone();
              app.selected_http_method =
                HttpMethod::from_str(&item.request.method.clone()).unwrap();
              if let Some(body) = item.request.body {
                if let Some(body_str) = body.raw {
                  // TODO - emit event
                  app.body_str = body_str;
                }
              }
              if let Some(headers) = item.request.header {
                let constructed_headers: Vec<(bool, String, String)> = headers
                  .into_iter()
                  .map(|h| (true, h.key, h.value))
                  .collect();
                app.headers = Rc::new(RefCell::from(constructed_headers));
              }
            }
          }
          CollectionItemOrFolder::Folder(folder) => {
            render_folder(ui, app, c, folder);
          }
        };
      }
    })
  })
}

fn render_folder(
  ui: &mut egui::Ui,
  app: &mut Gui,
  c: &Collection,
  f: CollectionFolder,
) -> InnerResponse<()> {
  ui.horizontal(|ui| {
    if ui.button("X").clicked() {
      let clicked_col_id = c.clone().info.id;
      let refresh_clone = app.collections.clone();
      let folder_for_worker = f.clone();
      tokio::spawn(async move {
        let _ = PostieApi::delete_collection_folder(clicked_col_id, folder_for_worker.name);
        Gui::refresh_collections(refresh_clone).await;
      });
    }
    if ui
      .collapsing(f.clone().name, |ui| {
        for f_item in f.clone().item {
          match f_item {
            CollectionItemOrFolder::Item(i) => ui.horizontal(|ui| {
              if ui.button("X").clicked() {
                let clicked_col_id = c.clone().info.id;
                let clicked_req_name = i.name.clone();
                let refresh_clone = app.collections.clone();
                let folder_for_worker = f.clone();
                tokio::spawn(async move {
                  let _ = PostieApi::delete_collection_request(
                    clicked_col_id,
                    folder_for_worker.name,
                    clicked_req_name,
                  )
                  .await;
                  Gui::refresh_collections(refresh_clone).await;
                });
              }
              if ui
                .selectable_value(
                  &mut app.selected_request.clone(),
                  Rc::new(RefCell::from(Some(i.clone().request))),
                  i.name.to_string(),
                )
                .clicked()
              {
                app.url = i.request.url.raw;
                app.selected_http_method = HttpMethod::from_str(&i.request.method.clone()).unwrap();
                app.body_str = i.request.body.and_then(|b| b.raw).unwrap_or_default();
                let constructed_headers: Vec<(bool, String, String)> = i
                  .request
                  .header
                  .map(|headers| {
                    headers
                      .into_iter()
                      .map(|h| (true, h.key, h.value))
                      .collect()
                  })
                  .unwrap_or_default(); // default empty vec if none
                app.headers = Rc::new(RefCell::from(constructed_headers));
              }
            }),
            CollectionItemOrFolder::Folder(f) => render_folder(ui, app, c, f),
          };
        }
      })
      .header_response
      .clicked()
    {}
  })
}
