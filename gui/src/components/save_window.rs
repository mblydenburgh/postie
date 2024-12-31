use std::sync::Arc;

use api::domain::collection::{CollectionFolder, CollectionItemOrFolder};

use crate::Gui;

pub fn save_window(gui: &mut Gui, ctx: &egui::Context) {
  if let Ok(mut save_window_open) = gui.save_window_open.try_write() {
      if *save_window_open == true {
            egui::Window::new("Save request")
                .open(&mut *save_window_open)
                .show(ctx, |ui| {
                    let collections_clone = Arc::clone(&gui.collections);
                    let collections = collections_clone.try_write().unwrap();
                    if let Some(collections) = &*collections {
                        let mut selected_collection = collections.first().unwrap();
                        let mut selected_request_folder = match selected_collection.item.first().unwrap() {
                            CollectionItemOrFolder::Folder(folder) => folder,
                            CollectionItemOrFolder::Item(_) => &CollectionFolder {
                                name: "Root".to_string(),
                                item: vec![],
                            }
                        };
                        egui::ComboBox::from_label("Collection to add request to")
                            .selected_text(selected_collection.info.name.clone())
                            .show_ui(ui, |ui| {
                                for collection in collections {
                                    ui.selectable_value(
                                        &mut selected_collection,
                                        collection,
                                        collection.info.name.clone(),
                                    );
                                }
                            });
                        egui::ComboBox::from_label("Folder to add request to")
                            .selected_text(selected_request_folder.name.clone())
                            .show_ui(ui, |ui| {
                                for item in &selected_collection.item {
                                    match item {
                                        CollectionItemOrFolder::Folder(folder) => {
                                            ui.selectable_value(
                                                &mut selected_request_folder,
                                                folder,
                                                folder.name.clone(),
                                            );
                                        }
                                        CollectionItemOrFolder::Item(_) => {}
                                    }
                                }
                            });
                        if ui.button("Save").clicked() {
                            println!("Saving request to collection: {:?}", selected_collection.info.name);
                            println!("Saving to folder: {:?}", selected_request_folder.name);
                            let active_tab = gui.active_tab.try_read().unwrap();
                            if let Some(active_tab) = &*active_tab {
                                println!("updating request with tab info: {:?}", active_tab);
                            }
                        }
                    }
                });
      }
  }
}
