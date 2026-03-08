use crate::{GuiState, Tab, ThreadSafeState};
use std::{collections::HashMap, str::FromStr as _, sync::Arc};

use crate::events;
use api::domain::{
  collection::{
    Collection, CollectionFolder, CollectionItem, CollectionItemOrFolder, CollectionRequest,
  },
  environment::EnvironmentFile,
  header::Headers,
  request::{DBRequest, HttpMethod},
  request_item::RequestHistoryItem,
  response::DBResponse,
  tab,
  ui::{self, ActiveWindow},
};
use egui::{InnerResponse, ScrollArea, SidePanel};
use tokio::sync::RwLock;

pub struct ContentSidePanel {
  selected_request: Option<CollectionRequest>,
  selected_environment: Option<EnvironmentFile>,
  selected_history_item: Option<RequestHistoryItem>,
}
impl ContentSidePanel {
  pub fn new() -> Self {
    Self {
      selected_request: None,
      selected_environment: None,
      selected_history_item: None,
    }
  }

  pub fn show(
    &mut self,
    ctx: &egui::Context,
    gui_state: &GuiState,
    worker_state: &ThreadSafeState,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    let window_mode = if let Ok(guard) = gui_state.active_window.try_read() {
      (*guard).clone()
    } else {
      ActiveWindow::COLLECTIONS
    };
    SidePanel::left("content_panel").show(ctx, |ui| match window_mode {
      ui::ActiveWindow::COLLECTIONS => {
        ScrollArea::vertical().show(ui, |ui| {
          ui.label("Collections");
          self.render_collections(
            ctx,
            ui,
            &worker_state.collections.clone(),
            &worker_state.tabs.clone(),
            event_tx,
          );
        });
      }
      ui::ActiveWindow::ENVIRONMENT => {
        self.render_environments(ui, &worker_state.environments);
      }
      ui::ActiveWindow::HISTORY => {
        self.render_history(
          ui,
          &worker_state.request_history_items.clone(),
          &worker_state.saved_requests.clone(),
          &worker_state.saved_responses.clone(),
          &worker_state.tabs.clone(),
          event_tx,
        );
      }
    });
  }

  fn render_collections(
    &mut self,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    collections: &Arc<RwLock<Vec<Collection>>>,
    tabs: &Arc<RwLock<HashMap<String, tab::Tab>>>,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    let collections_read = collections.try_read();
    if let Ok(guard) = collections_read {
      for c in guard.iter() {
        ui.horizontal(|ui| {
          self.render_context_menu(ui, tabs, &c, None, None, event_tx);
          ui.collapsing(c.info.name.clone(), |ui| {
            for i in c.item.clone() {
              match i {
                CollectionItemOrFolder::Item(item) => {
                  self.render_request(ui, ctx, &c, None, &item, tabs, event_tx);
                }
                CollectionItemOrFolder::Folder(folder) => {
                  self.render_collection_folder(ui, ctx, tabs, &c, &folder, event_tx);
                }
              };
            }
          })
        });
      }
    };
  }

  fn render_request(
    &mut self,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    collection: &Collection,
    folder: Option<&CollectionFolder>,
    item: &CollectionItem,
    tabs: &Arc<RwLock<HashMap<String, tab::Tab>>>,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    ui.horizontal(|ui| {
      self.render_context_menu(ui, tabs, collection, folder, Some(item), event_tx);

      let is_selected = self.selected_request.as_ref() == Some(&item.request);

      if ui.selectable_label(is_selected, &item.name).clicked() {
        self.selected_request = Some(item.request.clone());

        let _ = event_tx.try_send(events::GuiEvent::SelectRequest {
          col_id: collection.info.id.clone(),
          request: item.request.clone(),
        });

        ctx.request_repaint();
      }
    });
  }

  fn render_collection_folder(
    &mut self,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    tabs: &Arc<RwLock<HashMap<String, tab::Tab>>>,
    c: &Collection,
    f: &CollectionFolder,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    ui.horizontal(|ui| {
      self.render_context_menu(ui, tabs, c, Some(f), None, event_tx);

      ui.collapsing(f.name.clone(), |ui| {
        for f_item in &f.item {
          match f_item {
            CollectionItemOrFolder::Item(i) => {
              self.render_request(ui, ctx, c, Some(f), i, tabs, event_tx);
            }
            CollectionItemOrFolder::Folder(sub_folder) => {
              self.render_collection_folder(ui, ctx, tabs, c, sub_folder, event_tx);
            }
          }
        }
      });
    });
  }

  fn render_environments(
    &mut self,
    ui: &mut egui::Ui,
    environments: &Arc<RwLock<Vec<EnvironmentFile>>>,
  ) {
    ScrollArea::vertical().show(ui, |ui| {
      ui.label("Environments");
      if let Ok(envs) = environments.try_read() {
        for env in envs.iter() {
          ui.selectable_value(
            &mut self.selected_environment,
            Some(env.clone()),
            env.name.to_string(),
          );
        }
      }
    });
  }

  fn render_history(
    &mut self,
    ui: &mut egui::Ui,
    history_items: &Arc<RwLock<Vec<RequestHistoryItem>>>,
    saved_requests: &Arc<RwLock<HashMap<String, DBRequest>>>,
    saved_responses: &Arc<RwLock<HashMap<String, DBResponse>>>,
    tabs: &Arc<RwLock<HashMap<String, tab::Tab>>>,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    ScrollArea::vertical().show(ui, |ui| {
      ui.label("History");
      let history_items = history_items.try_read().unwrap();
      let request_clone = saved_requests.try_read().unwrap();
      let response_clone = saved_responses.try_read().unwrap();
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
            &mut self.selected_history_item,
            Some(item.clone()),
            format!("{:?}", req_name), // TODO - create function to get name
          )
          .clicked()
        {
          /*
          When clicked,
          1. Check tabs for matching url
          2. If match, set active tab to that id
          3. If no match, create new tab and set as active
           */
          // TODO - replace url, method
          let historical_request = request_clone.get(&item.request_id).unwrap();
          let historical_response = response_clone.get(&item.response_id);
          let mut tabs_lock = tabs.try_write().unwrap();
          let tab_match = tabs_lock.iter().find(|t| t.1.url == historical_request.url);

          match tab_match {
            Some(t) => {
              println!("matching tab found, setting active");
              event_tx
                .try_send(events::GuiEvent::SetActiveTab(t.0.clone()))
                .unwrap()
            }
            None => {
              println!("no matching tab found, creating new");
              let id = uuid::Uuid::new_v4();
              let new_tab = Tab {
                id,
                method: HttpMethod::from_str(&historical_request.method).unwrap(),
                url: historical_request.url.clone(),
                req_body: historical_request.body.clone().unwrap_or_default(),
                req_headers: Headers(historical_request.headers.clone()),
                res_status: None,
                res_body: match historical_response {
                  Some(r) => r.body.clone().unwrap_or_default(),
                  None => String::new(),
                },
                res_headers: match historical_response {
                  Some(r) => Headers(r.headers.clone()),
                  None => {
                    let headers: Vec<(String, String)> = vec![];
                    Headers::from_iter(headers)
                  }
                },
              };
              tabs_lock.insert(new_tab.id.clone().to_string(), new_tab.clone());
              event_tx
                .try_send(events::GuiEvent::SetActiveTab(
                  new_tab.id.clone().to_string(),
                ))
                .unwrap();
            }
          }
        }
      }
    });
  }

  fn render_context_menu(
    &mut self,
    ui: &mut egui::Ui,
    tabs: &Arc<RwLock<HashMap<String, tab::Tab>>>,
    col: &Collection,
    fol: Option<&CollectionFolder>,
    req: Option<&CollectionItem>,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) -> InnerResponse<Option<()>> {
    ui.menu_button("...", |ui| {
      ui.menu_button("New", |ui| {
        if (ui.button("Request")).clicked() {
          println!("adding request");
          let mut tabs = tabs.try_write().unwrap();
          let new_tab = Tab::default();
          tabs.insert(new_tab.id.clone().to_string(), new_tab);
        }
        if (ui.button("Folder")).clicked() {
          println!("adding folder");
        }
      });
      if ui.button("Delete").clicked() {
        match (fol, req) {
          (Some(f), Some(r)) => {
            event_tx
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
            event_tx
              .try_send(events::GuiEvent::RemoveCollectionFolder(
                events::RemoveCollectionItemPayload {
                  id: col.info.id.clone(),
                  name: f.name.clone(),
                },
              ))
              .unwrap();
          }
          (None, Some(r)) => {
            event_tx
              .try_send(events::GuiEvent::RemoveCollectionRequest(
                events::RemoveCollectionItemPayload {
                  id: col.info.id.clone(),
                  name: r.name.clone(),
                },
              ))
              .unwrap();
          }
          (None, None) => {
            event_tx
              .try_send(events::GuiEvent::RemoveCollection(col.info.id.clone()))
              .unwrap();
          }
        }
        ui.close();
      }
    })
  }
}
