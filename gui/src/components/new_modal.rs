use std::{thread, time};

use api::{
  domain::collection::{Collection, CollectionFolder, CollectionItemOrFolder},
  domain::ui::NewWindowMode,
};

use crate::{events, GuiState, ThreadSafeState};

pub struct NewWindow {
  pub window_mode: NewWindowMode,
  pub is_open: bool,
  pub new_name: String,
}
impl NewWindow {
  pub fn new() -> Self {
    Self {
      window_mode: NewWindowMode::COLLECTION,
      is_open: false,
      new_name: String::new(),
    }
  }

  pub fn show(
    &mut self,
    ctx: &egui::Context,
    gui_state: &GuiState,
    worker_state: &ThreadSafeState,
    event_tx: &tokio::sync::mpsc::Sender<events::GuiEvent>,
  ) {
    if let Ok(mut new_window_open) = gui_state.new_window_open.try_write() {
      if *new_window_open {
        egui::Window::new("Create new")
          .open(&mut new_window_open)
          .show(ctx, |ui| {
            ui.horizontal(|ui| {
              ui.label("Enter name: ");
              ui.text_edit_singleline(&mut self.new_name);
              match self.window_mode {
                NewWindowMode::FOLDER => {
                  let collections = &worker_state.collections.try_read().unwrap().clone();
                  let selected_collection = &mut gui_state.selected_save_window_collection.clone();
                  egui::ComboBox::from_label("Collection to add folder to")
                    .selected_text(
                      selected_collection
                        .as_ref()
                        .map_or("Select a collection".to_string(), |col| {
                          col.info.name.clone()
                        }),
                    )
                    .show_ui(ui, |ui| {
                      for col in collections {
                        ui.selectable_value(
                          selected_collection,
                          Some(col.clone()),
                          col.info.name.clone(),
                        );
                      }
                    });
                }
                _ => {}
              }
              if ui.button("Save").clicked() {
                match self.window_mode {
                  NewWindowMode::COLLECTION => {
                    let _collections_for_worker = worker_state.collections.clone();
                    let tx_clone = event_tx.clone();
                    let tx_clone2 = event_tx.clone();
                    let name_for_worker = self.new_name.clone();
                    let _ = tokio::spawn(async move {
                      tx_clone
                        .try_send(events::GuiEvent::NewCollection(Some(name_for_worker)))
                        .unwrap();
                    });
                    _ = tokio::spawn(async move {
                      let sleep = time::Duration::from_millis(50);
                      thread::sleep(sleep);
                      tx_clone2
                        .try_send(events::GuiEvent::RefreshCollections(None))
                        .unwrap();
                    });
                  }
                  NewWindowMode::ENVIRONMENT => {
                    let _envs_for_worker = worker_state.environments.clone();
                    let tx_clone = event_tx.clone();
                    let tx_clone2 = event_tx.clone();
                    let name_clone2 = self.new_name.clone();
                    let _ = tokio::spawn(async move {
                      tx_clone
                        .try_send(events::GuiEvent::NewEnvironment(Some(name_clone2)))
                        .unwrap();
                    });
                    // TODO - why have two threads?
                    _ = tokio::spawn(async move {
                      let sleep = time::Duration::from_millis(50);
                      thread::sleep(sleep);
                      tx_clone2
                        .try_send(events::GuiEvent::RefreshEnvironments())
                        .unwrap();
                    });
                  }
                  NewWindowMode::FOLDER => {
                    //call to update collection with new folder
                    let collection_for_worker =
                      gui_state.selected_save_window_collection.clone().unwrap();
                    let _collections_for_worker = worker_state.collections.clone();
                    let name_for_worker = self.new_name.clone();

                    let tx_clone = event_tx.clone();
                    let _ = tokio::spawn(async move {
                      // take selected collection, add new folder to the top
                      // `item` field as with no requests
                      let mut collection_items = collection_for_worker.item;
                      let new_folder = CollectionItemOrFolder::Folder(CollectionFolder {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: name_for_worker,
                        item: vec![],
                      });
                      collection_items.push(new_folder);
                      let updated_collection: Collection = Collection {
                        info: collection_for_worker.info,
                        item: collection_items,
                        auth: collection_for_worker.auth,
                      };

                      tx_clone
                        .try_send(events::GuiEvent::SaveCollection(updated_collection))
                        .unwrap();

                      // refresh collections with new folder now added
                      let sleep = time::Duration::from_millis(50);
                      thread::sleep(sleep);
                      tx_clone
                        .try_send(events::GuiEvent::RefreshCollections(None))
                        .unwrap();
                    });
                  }
                }
              }
            })
          });
      }
    }
  }
}
