use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};

use crate::{events, Gui};
use api::domain::{
  collection::{Collection, CollectionFolder, CollectionItem, CollectionItemOrFolder},
  request::{DBRequest, HttpMethod},
  response::ResponseData,
  ui,
};
use egui::{
  scroll_area::ScrollAreaOutput, CollapsingResponse, InnerResponse, ScrollArea, SidePanel,
};

pub fn content_side_panel(app: &mut Gui, ctx: &egui::Context) {
  let collections_guard = app.worker_state.collections.try_read().unwrap().clone();
  let active_winow_guard = app.gui_state.active_window.try_read().unwrap().clone();
  SidePanel::left("content_panel").show(ctx, |ui| match active_winow_guard {
    ui::ActiveWindow::COLLECTIONS => {
      ScrollArea::vertical().show(ui, |ui| {
        ui.label("Collections");
        let cols = collections_guard;
        for c in cols {
          render_collection(ui, app, &c);
        }
      });
    }
    ui::ActiveWindow::ENVIRONMENT => {
      render_environments(ui, app);
    }
    ui::ActiveWindow::HISTORY => {
      render_history(ui, app);
    }
  });
}

fn render_collection(
  ui: &mut egui::Ui,
  app: &mut Gui,
  c: &Collection,
) -> InnerResponse<CollapsingResponse<()>> {
  ui.horizontal(|ui| {
    render_context_menu(ui, app, c, None, None);
    ui.collapsing(c.info.name.clone(), |ui| {
      for i in c.item.clone() {
        match i {
          CollectionItemOrFolder::Item(item) => {
            ui.horizontal(|ui| {
              render_context_menu(ui, app, c, None, Some(&item));
              if ui
                .selectable_value(
                  &mut app.gui_state.selected_request.clone(),
                  Rc::new(RefCell::from(Some(item.clone().request))),
                  item.name.to_string(),
                )
                .clicked()
              {
                app.gui_state.url = item.request.url.raw.clone();
                app.gui_state.selected_http_method =
                  HttpMethod::from_str(&item.request.method.clone()).unwrap();
                if let Some(body) = item.request.body {
                  if let Some(body_str) = body.raw {
                    app.gui_state.body_str = body_str;
                  }
                }
                if let Some(headers) = item.request.header {
                  let constructed_headers: Vec<(bool, String, String)> = headers
                    .into_iter()
                    .map(|h| (true, h.key, h.value))
                    .collect();
                  app.gui_state.headers = Rc::new(RefCell::from(constructed_headers));
                }
              }
            });
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
    render_context_menu(ui, app, c, Some(&f), None);
    if ui
      .collapsing(f.clone().name, |ui| {
        for f_item in f.clone().item {
          match f_item {
            CollectionItemOrFolder::Item(i) => ui.horizontal(|ui| {
              render_context_menu(ui, app, c, Some(&f), Some(&i));
              if ui
                .selectable_value(
                  &mut app.gui_state.selected_request.clone(),
                  Rc::new(RefCell::from(Some(i.clone().request))),
                  i.name.to_string(),
                )
                .clicked()
              {
                app.gui_state.url = i.request.url.raw;
                app.gui_state.selected_http_method =
                  HttpMethod::from_str(&i.request.method.clone()).unwrap();
                app.gui_state.body_str = i.request.body.and_then(|b| b.raw).unwrap_or_default();
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
                app.gui_state.headers = Rc::new(RefCell::from(constructed_headers));
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

fn render_environments(ui: &mut egui::Ui, app: &mut Gui) -> ScrollAreaOutput<()> {
  ScrollArea::vertical().show(ui, |ui| {
    ui.label("Environments");
    let envs_clone = Arc::clone(&app.worker_state.environments);
    let envs = envs_clone.try_write().unwrap();
    let env_vec = &*envs;
    for env in env_vec {
      ui.selectable_value(
        &mut app.gui_state.selected_environment,
        Rc::new(RefCell::from(env.clone())),
        env.name.to_string(),
      );
    }
  })
}

fn render_history(ui: &mut egui::Ui, app: &mut Gui) -> ScrollAreaOutput<()> {
  ScrollArea::vertical().show(ui, |ui| {
    ui.label("History");
    let history_items_clone = Arc::clone(&app.worker_state.request_history_items);
    let history_items = history_items_clone.try_write().unwrap();
    let request_clone = app.worker_state.saved_requests.try_write().unwrap();
    let item_vec = &*history_items;
    for item in item_vec {
      let history_reqs = &request_clone;
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
          &mut app.gui_state.selected_history_item,
          Rc::new(RefCell::from(Some(item.clone()))),
          format!("{:?}", req_name), // TODO - create function to get name
        )
        .clicked()
      {
        // TODO - replace url, method
        let responses_clone = app.worker_state.saved_responses.try_write().unwrap();
        let responses = responses_clone;
        let historical_request = request_clone.get(&item.request_id).unwrap();
        let historical_response = responses.get(&item.response_id).unwrap();
        app.gui_state.url = historical_request.url.clone();
        app.gui_state.selected_http_method =
          HttpMethod::from_str(&historical_request.method).unwrap();
        match &historical_request.body {
          Some(body_json) => {
            app.gui_state.body_str = body_json.to_string();
          }
          None => app.gui_state.body_str = String::from(""),
        }
        let ui_response_clone = app.worker_state.response.clone();
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
                ResponseData::TEXT(body.clone().into())
              }
            };
            *ui_response_guard = Some(parsed_body)
          }
          None => *ui_response_guard = None,
        }
      }
    }
  })
}

fn render_context_menu(
  ui: &mut egui::Ui,
  app: &mut Gui,
  col: &Collection,
  fol: Option<&CollectionFolder>,
  req: Option<&CollectionItem>,
) -> InnerResponse<Option<()>> {
  ui.menu_button("...", |ui| {
    ui.menu_button("New", |ui| {
      if (ui.button("Request")).clicked() {
        println!("adding request");
      }
      if (ui.button("Folder")).clicked() {
        println!("adding folder");
      }
    });
    if ui.button("Delete").clicked() {
      match (fol, req) {
        (Some(f), Some(r)) => {
          app
            .event_tx
            .try_send(events::GuiEvent::RemoveCollectionFolderRequest(
              events::RemoveCollectionRequestPayload {
                col_id: col.info.id.clone(),
                folder_name: f.name.clone(),
                req_name: r.name.clone(),
              },
            ))
            .unwrap();
        }
        (Some(f), None) => {
          app
            .event_tx
            .try_send(events::GuiEvent::RemoveCollectionFolder(
              events::RemoveCollectionItemPayload {
                id: col.info.id.clone(),
                name: f.name.clone(),
              },
            ))
            .unwrap();
        }
        (None, Some(r)) => {
          app
            .event_tx
            .try_send(events::GuiEvent::RemoveCollectionRequest(
              events::RemoveCollectionItemPayload {
                id: col.info.id.clone(),
                name: r.name.clone(),
              },
            ))
            .unwrap();
        }
        (None, None) => {
          app
            .event_tx
            .try_send(events::GuiEvent::RemoveCollection(col.info.id.clone()))
            .unwrap();
        }
      }
      ui.close_menu();
    }
  })
}
