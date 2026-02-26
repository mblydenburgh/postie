use std::collections::HashMap;
use std::{ops::Deref, sync::Arc};

use api::domain::tab::Tab;
use api::domain::ui::{ImportMode, NewWindowMode};
use egui::TopBottomPanel;

use api::domain::{tab, ui};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use crate::events::GuiEvent;
use crate::{events, GuiState, ThreadSafeState};

pub struct MenuPanel {}
impl MenuPanel {
  pub fn new() -> Self {
    Self {}
  }

  pub fn show(
    &self,
    ctx: &egui::Context,
    event_tx: &Sender<GuiEvent>,
    gui_state: &GuiState,
    worker_state: &ThreadSafeState,
  ) {
    self.render_menu_panel(
      ctx,
      &gui_state.new_window_open,
      &gui_state.new_window_mode,
      &worker_state.tabs,
      &gui_state.save_window_open,
      &gui_state.import_window_open,
      &gui_state.import_mode,
      &worker_state.is_requesting,
      &worker_state.res_status,
    );
    self.render_tabs_panel(ctx, event_tx, &worker_state.tabs);
  }

  fn render_menu_panel(
    &self,
    ctx: &egui::Context,
    new_window_open: &RwLock<bool>,
    new_window_mode: &RwLock<NewWindowMode>,
    tabs: &Arc<RwLock<HashMap<String, Tab>>>,
    save_window_open: &RwLock<bool>,
    import_window_open: &RwLock<bool>,
    import_mode: &RwLock<ImportMode>,
    is_requesting: &Arc<RwLock<Option<bool>>>,
    res_status: &Arc<RwLock<String>>,
  ) {
    TopBottomPanel::top("menu_panel").show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.menu_button("Menu", |ui| {
          ui.menu_button("New", |ui| {
            if ui.button("Request").clicked() {
              let mut tabs = tabs.try_write().unwrap();
              let new_tab = tab::Tab::default();
              tabs.insert(new_tab.id.clone().to_string(), new_tab);
            }
            if ui.button("Collection").clicked() {
              if let Ok(mut new_model_open) = new_window_open.try_write() {
                *new_model_open = true;
              }
              if let Ok(mut new_model_mode) = new_window_mode.try_write() {
                *new_model_mode = ui::NewWindowMode::COLLECTION;
              }
            };
            if ui.button("Collection Folder").clicked() {
              if let Ok(mut new_model_open) = new_window_open.try_write() {
                *new_model_open = true;
              }
              if let Ok(mut new_model_mode) = new_window_mode.try_write() {
                *new_model_mode = ui::NewWindowMode::FOLDER;
              }
            };
            if ui.button("Environment").clicked() {
              if let Ok(mut new_model_open) = new_window_open.try_write() {
                *new_model_open = true;
              }
              if let Ok(mut new_model_mode) = new_window_mode.try_write() {
                *new_model_mode = ui::NewWindowMode::ENVIRONMENT;
              }
            };
          });
          ui.menu_button("Save", |ui| {
            if ui.button("Request").clicked() {
              if let Ok(mut save_window_open) = save_window_open.try_write() {
                *save_window_open = true;
              }
            }
          });
          ui.menu_button("Import", |ui| {
            if ui.button("Collection").clicked() {
              if let Ok(mut import_open) = import_window_open.try_write() {
                *import_open = true;
              }
              if let Ok(mut import_mode) = import_mode.try_write() {
                *import_mode = ui::ImportMode::COLLECTION;
              }
            };
            if ui.button("Environment").clicked() {
              if let Ok(mut import_open) = import_window_open.try_write() {
                *import_open = true;
              }
              if let Ok(mut import_mode) = import_mode.try_write() {
                *import_mode = ui::ImportMode::ENVIRONMENT;
              }
            };
          });
          ui.menu_button("Export", |ui| {
            if ui.button("Collection").clicked() {
              ui.close_menu();
            };
            if ui.button("Environment").clicked() {
              ui.close_menu();
            };
          });
        });
        let is_requesting_lock = is_requesting.try_read();
        if is_requesting_lock.is_ok() {
          if let Ok(is_requesting) = is_requesting_lock {
            match is_requesting.deref() {
              Some(r) => {
                if *r {
                  ui.label("Requesting...");
                } else {
                  let response_status_lock = res_status.try_read();
                  if response_status_lock.is_ok() {
                    if let Ok(response_status) = response_status_lock {
                      ui.label(response_status.deref());
                    }
                  }
                }
              }
              None => {}
            }
          }
        }
      });
    });
  }

  fn render_tabs_panel(
    &self,
    ctx: &egui::Context,
    event_tx: &Sender<GuiEvent>,
    tabs: &Arc<RwLock<HashMap<String, Tab>>>,
  ) {
    TopBottomPanel::top("tabs panel").show(ctx, |ui| {
      let tabs_clone = Arc::clone(&tabs);
      let tabs = tabs_clone.try_read().unwrap();
      ui.horizontal(|ui| {
        for tab in &*tabs {
          let name = if tab.1.url.is_empty() {
            "Unsent Request".to_string()
          } else {
            tab.1.url.clone()
          };
          ui.horizontal(|ui| {
            let id = tab.1.id;
            if ui.button("X").clicked() {
              event_tx.try_send(events::GuiEvent::RemoveTab(id)).unwrap();
            }
            if ui.button(&name).clicked() {
              event_tx
                .try_send(events::GuiEvent::SetActiveTab(id.to_string()))
                .unwrap();
            }
          });
        }
      });
    });
  }
}
