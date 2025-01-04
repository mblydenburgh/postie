use std::{thread, time};

use api::{
    domain::collection::{Collection, CollectionFolder, CollectionItemOrFolder},
    domain::ui::NewWindowMode,
    PostieApi,
};
use uuid::Uuid;

use crate::Gui;

pub fn new_modal(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(mut new_window_open) = gui.new_window_open.try_write() {
        if *new_window_open {
            egui::Window::new("Create new")
                .open(&mut new_window_open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Enter name: ");
                        ui.text_edit_singleline(&mut gui.new_name);
                        if let Ok(new_window_mode) = gui.new_window_mode.try_read() {
                            match *new_window_mode {
                                NewWindowMode::FOLDER => {
                                    let collections = &gui.collections.try_read().unwrap().clone();
                                    let selected_collection =
                                        &mut gui.selected_save_window_collection;
                                    if let Some(cols) = collections {
                                        egui::ComboBox::from_label("Collection to add folder to")
                                            .selected_text(
                                                selected_collection.as_ref().map_or(
                                                    "Select a collection".to_string(),
                                                    |col| col.info.name.clone(),
                                                ),
                                            )
                                            .show_ui(ui, |ui| {
                                                for col in cols {
                                                    ui.selectable_value(
                                                        selected_collection,
                                                        Some(col.clone()),
                                                        col.info.name.clone(),
                                                    );
                                                }
                                            });
                                    }
                                }
                                _ => {}
                            }
                            if ui.button("Save").clicked() {
                                match *new_window_mode {
                                    NewWindowMode::COLLECTION => {
                                        let blank_collection =
                                            api::domain::collection::Collection {
                                                info: api::domain::collection::CollectionInfo {
                                                    id: Uuid::new_v4().to_string(),
                                                    name: gui.new_name.clone(),
                                                    description: None,
                                                },
                                                item: vec![],
                                                auth: None,
                                            };
                                        let collections_for_worker = gui.collections.clone();
                                        let _ = tokio::spawn(async move {
                                            PostieApi::save_collection(blank_collection)
                                                .await
                                                .unwrap();
                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(50);
                                            thread::sleep(sleep);
                                            Gui::refresh_collections(collections_for_worker).await;
                                        });
                                    }
                                    NewWindowMode::ENVIRONMENT => {
                                        let blank_env = api::domain::environment::EnvironmentFile {
                                            id: Uuid::new_v4().to_string(),
                                            name: gui.new_name.clone(),
                                            values: None,
                                        };
                                        let envs_for_worker = gui.environments.clone();
                                        let _ = tokio::spawn(async move {
                                            PostieApi::save_environment(blank_env)
                                                .await
                                                .unwrap();
                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(50);
                                            thread::sleep(sleep);
                                            Gui::refresh_environments(envs_for_worker).await;
                                        });
                                    }
                                    NewWindowMode::FOLDER => {
                                        //call to update collection with new folder
                                        let collection_for_worker =
                                            gui.selected_save_window_collection.clone().unwrap();
                                        let collections_for_worker = gui.collections.clone();
                                        let name_for_worker = gui.new_name.clone();

                                        let _ = tokio::spawn(async move {
                                            // take selected collection, add new folder to the top
                                            // `item` field as with no requests
                                            let mut collection_items = collection_for_worker.item;
                                            let new_folder =
                                                CollectionItemOrFolder::Folder(CollectionFolder {
                                                    name: name_for_worker,
                                                    item: vec![],
                                                });
                                            collection_items.push(new_folder);
                                            let updated_collection: Collection = Collection {
                                                info: collection_for_worker.info,
                                                item: collection_items,
                                                auth: collection_for_worker.auth,
                                            };

                                            PostieApi::save_collection(updated_collection)
                                                .await
                                                .unwrap();

                                            // refresh collections with new folder now added
                                            let sleep = time::Duration::from_millis(50);
                                            thread::sleep(sleep);
                                            Gui::refresh_collections(collections_for_worker).await;
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
