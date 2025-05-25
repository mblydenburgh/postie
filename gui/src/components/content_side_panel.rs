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
use tokio::sync::RwLock;

pub fn content_side_panel(gui: &mut Gui, ctx: &egui::Context) {
  if let Ok(active_window) = gui.active_window.try_read() {
    let collections_clone = Arc::clone(&gui.collections);
    let selected_req_clone = Rc::clone(&gui.selected_request);
    let mut url_clone = gui.url.clone();
    let mut selected_method_clone = gui.selected_http_method.clone();
    let mut req_headers_clone = Rc::clone(&gui.headers);
    let mut req_body_clone = gui.body_str.clone();
    SidePanel::left("content_panel").show(ctx, |ui| match *active_window {
      ui::ActiveWindow::COLLECTIONS => {
        ScrollArea::vertical().show(ui, |ui| {
          ui.label("Collections");
          let collections = collections_clone.try_write().unwrap();
          if let Some(cols) = &*collections {
            for c in cols {
              let c_clone = c.clone();
              render_collection(
                ui,
                &selected_req_clone,
                &mut url_clone,
                &mut selected_method_clone,
                &mut req_headers_clone,
                &mut req_body_clone,
                &c_clone,
                &collections_clone,
              );
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
                gui.selected_http_method =
                  HttpMethod::from_str(&historical_request.method).unwrap();
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
}

fn render_collection(
  ui: &mut egui::Ui,
  selected_req: &Rc<RefCell<Option<api::domain::collection::CollectionRequest>>>,
  url: &mut String,
  selected_method: &mut HttpMethod,
  req_headers: &mut Rc<RefCell<Vec<(bool, String, String)>>>,
  req_body: &mut String,
  c: &Collection,
  cols: &Arc<RwLock<Option<Vec<api::domain::collection::Collection>>>>,
) -> InnerResponse<CollapsingResponse<()>> {
  ui.horizontal(|ui| {
    if ui.button("X").clicked() {
      let clicked_id = c.info.id.clone();
      // call to delete collection by id, refresh collections for ui
      let refresh_clone = cols.clone();
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
                &mut selected_req.clone(),
                Rc::new(RefCell::from(Some(item.clone().request))),
                item.name.to_string(),
              )
              .clicked()
            {
              // TODO - emit channel event to update gui fields
              *url = item.request.url.raw.clone();
              *selected_method = HttpMethod::from_str(&item.request.method.clone()).unwrap();
              if let Some(body) = item.request.body {
                if let Some(body_str) = body.raw {
                  // TODO - emit event
                  *req_body = body_str;
                }
              }
              if let Some(headers) = item.request.header {
                let constructed_headers: Vec<(bool, String, String)> = headers
                  .into_iter()
                  .map(|h| (true, h.key, h.value))
                  .collect();
                // TODO - emit event
                *req_headers = Rc::new(RefCell::from(constructed_headers));
              }
            }
          }
          CollectionItemOrFolder::Folder(folder) => {
            let selected_req_clone = Rc::clone(&selected_req);
            let mut url_clone = url.clone();
            let mut selected_method_clone = selected_method.clone();
            let mut req_headers_clone = Rc::clone(&req_headers);
            let mut req_body_clone = req_body.clone();
            render_folder(
              ui,
              &selected_req_clone,
              &mut url_clone,
              &mut selected_method_clone,
              &mut req_headers_clone,
              &mut req_body_clone,
              c,
              folder,
              &Arc::clone(&cols),
            );
          }
        };
      }
    })
  })
}

fn render_folder(
  ui: &mut egui::Ui,
  selected_req: &Rc<RefCell<Option<api::domain::collection::CollectionRequest>>>,
  url: &mut String,
  selected_method: &mut HttpMethod,
  req_headers: &mut Rc<RefCell<Vec<(bool, String, String)>>>,
  req_body: &mut String,
  c: &Collection,
  f: CollectionFolder,
  cols: &Arc<RwLock<Option<Vec<api::domain::collection::Collection>>>>,
) -> InnerResponse<()> {
  ui.horizontal(|ui| {
    if ui.button("X").clicked() {
      let clicked_col_id = c.clone().info.id;
      let refresh_clone = cols.clone();
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
                let refresh_clone = cols.clone();
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
                  &mut selected_req.clone(),
                  Rc::new(RefCell::from(Some(i.clone().request))),
                  i.name.to_string(),
                )
                .clicked()
              {
                println!("clicked");
                // TODO - emit channel event to update gui fields
                *url = i.request.url.raw.clone();
                *selected_method = HttpMethod::from_str(&i.request.method.clone()).unwrap();
                if let Some(body) = i.request.body {
                  if let Some(body_str) = body.raw {
                    // TODO - emit event
                    *req_body = body_str;
                  }
                }
                if let Some(headers) = i.request.header {
                  let constructed_headers: Vec<(bool, String, String)> = headers
                    .into_iter()
                    .map(|h| (true, h.key, h.value))
                    .collect();
                  // TODO - emit event
                  *req_headers = Rc::new(RefCell::from(constructed_headers));
                }
              }
            }),
            CollectionItemOrFolder::Folder(f) => {
              let selected_req_clone = Rc::clone(&selected_req);
              let mut url_clone = url.clone();
              let mut selected_method_clone = selected_method.clone();
              let mut req_headers_clone = Rc::clone(&req_headers);
              let mut req_body_clone = req_body.clone();
              render_folder(
                ui,
                &selected_req_clone,
                &mut url_clone,
                &mut selected_method_clone,
                &mut req_headers_clone,
                &mut req_body_clone,
                c,
                f,
                cols,
              )
            }
          };
        }
      })
      .header_response
      .clicked()
    {}
  })
}
