use std::{thread, time};

use api::{
  domain::collection::{Collection, CollectionFolder, CollectionItemOrFolder},
  domain::ui::NewWindowMode,
};

use crate::events;
use crate::Gui;

pub fn new_modal(gui: &mut Gui, ctx: &egui::Context) {
  if let Ok(mut new_window_open) = gui.gui_state.new_window_open.try_write() {
    if *new_window_open {
      egui::Window::new("Create new")
        .open(&mut new_window_open)
        .show(ctx, |ui| {
          ui.horizontal(|ui| {
            ui.label("Enter name: ");
            ui.text_edit_singleline(&mut gui.gui_state.new_name);
            if let Ok(new_window_mode) = gui.gui_state.new_window_mode.try_read() {
              match *new_window_mode {
                NewWindowMode::FOLDER => {
                  let collections = &gui.worker_state.collections.try_read().unwrap().clone();
                  let selected_collection = &mut gui.gui_state.selected_save_window_collection;
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
                match *new_window_mode {
                  NewWindowMode::COLLECTION => {
                    let collections_for_worker = gui.worker_state.collections.clone();
                    let tx_clone = gui.event_tx.clone();
                    let tx_clone2 = gui.event_tx.clone();
                    let name_for_worker = gui.gui_state.new_name.clone();
                    let _ = tokio::spawn(async move {
                      tx_clone.send(events::GuiEvent::NewCollection(Some(name_for_worker)));
                    });
                    _ = tokio::spawn(async move {
                      let sleep = time::Duration::from_millis(50);
                      thread::sleep(sleep);
                      tx_clone2.send(events::GuiEvent::RefreshCollections());
                    });
                  }
                  NewWindowMode::ENVIRONMENT => {
                    let envs_for_worker = gui.worker_state.environments.clone();
                    let tx_clone = gui.event_tx.clone();
                    let tx_clone2 = gui.event_tx.clone();
                    let name_clone2 = gui.gui_state.new_name.clone();
                    let _ = tokio::spawn(async move {
                      tx_clone.send(events::GuiEvent::NewEnvironment(Some(name_clone2)));
                    });
                    _ = tokio::spawn(async move {
                      let sleep = time::Duration::from_millis(50);
                      thread::sleep(sleep);
                      tx_clone2.send(events::GuiEvent::RefreshEnvironments());
                    });
                  }
                  NewWindowMode::FOLDER => {
                    //call to update collection with new folder
                    let collection_for_worker = gui
                      .gui_state
                      .selected_save_window_collection
                      .clone()
                      .unwrap();
                    let collections_for_worker = gui.worker_state.collections.clone();
                    let name_for_worker = gui.gui_state.new_name.clone();

                    let tx_clone = gui.event_tx.clone();
                    let _ = tokio::spawn(async move {
                      // take selected collection, add new folder to the top
                      // `item` field as with no requests
                      let mut collection_items = collection_for_worker.item;
                      let new_folder = CollectionItemOrFolder::Folder(CollectionFolder {
                        name: name_for_worker,
                        item: vec![],
                      });
                      collection_items.push(new_folder);
                      let updated_collection: Collection = Collection {
                        info: collection_for_worker.info,
                        item: collection_items,
                        auth: collection_for_worker.auth,
                      };

                      // TODO pass updated_collection
                      tx_clone.send(events::GuiEvent::SaveCollection());

                      // refresh collections with new folder now added
                      let sleep = time::Duration::from_millis(50);
                      thread::sleep(sleep);
                      tx_clone.send(events::GuiEvent::RefreshCollections());
                    });
                  }
                }
              }
            }
          })
        });
    }
  }
}
