use std::str::FromStr;

use api::{
    domain::{
        collection::CollectionItemOrFolder,
        environment::EnvironmentFile,
        request::{HttpRequest, RequestBody},
    },
    PostieApi,
};
use uuid::Uuid;

use crate::Gui;

pub fn save_window(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(mut save_window_open) = gui.save_window_open.try_write() {
        if *save_window_open {
            egui::Window::new("Save request")
                .open(&mut save_window_open)
                .show(ctx, |ui| {
                    let collections = &gui.collections.try_read().unwrap().clone();

                    // Use struct fields directly
                    let selected_collection = &mut gui.selected_save_window_collection; // This should be Option<Collection>
                    let selected_folder = &mut gui.selected_save_window_folder; // This should be Option<String>

                    if let Some(cols) = collections {
                        egui::ComboBox::from_label("Collection to add request to")
                            .selected_text(
                                selected_collection
                                    .as_ref()
                                    .map_or("Select a collection".to_string(), |col| {
                                        col.info.name.clone()
                                    }),
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

                        if let Some(collection) = &selected_collection {
                            egui::ComboBox::from_label("Folder to add request to")
                                .selected_text(
                                    selected_folder
                                        .as_ref()
                                        .map_or("Select a folder".to_string(), |f| f.clone()),
                                )
                                .show_ui(ui, |ui| {
                                    for item in collection.item.iter() {
                                        match item {
                                            CollectionItemOrFolder::Folder(folder) => {
                                                ui.selectable_value(
                                                    selected_folder,
                                                    Some(folder.name.clone()),
                                                    folder.name.clone(),
                                                );
                                            }
                                            CollectionItemOrFolder::Item(_) => {}
                                        }
                                    }
                                });
                        }

                        if ui.button("Save").clicked() {
                            println!("Saving request to collection");
                            println!("Saving to folder: {:?}", selected_folder);

                            let active_tab = gui.active_tab.try_read().unwrap().clone();
                            if let Some(active_tab) = active_tab {
                                println!("updating request with tab info: {:?}", active_tab);

                                // Use a copy of the needed variables and move them into the async block
                                let selected_collection_id =
                                    selected_collection.clone().unwrap().info.id.clone();

                                let selected_folder_value_for_worker = selected_folder.clone();
                                let collections_for_worker = gui.collections.clone();
                                tokio::spawn(async move {
                                    let headers: Vec<(String, String)> = active_tab
                                        .req_headers
                                        .into_iter()
                                        .map(|header| (header.key.clone(), header.value.clone()))
                                        .collect();

                                    let request: HttpRequest = HttpRequest {
                                        tab_id: Uuid::from_str(&active_tab.id).unwrap(),
                                        id: Uuid::new_v4(),
                                        name: None,
                                        method: active_tab.method.clone(),
                                        url: active_tab.url.clone(),
                                        headers: Some(headers),
                                        body: serde_json::from_str(&active_tab.req_body)
                                            .ok()
                                            .map(RequestBody::JSON),
                                        environment: EnvironmentFile {
                                            id: "".into(),
                                            name: "".into(),
                                            values: None,
                                        },
                                    };

                                    let _ = PostieApi::add_request_to_collection(
                                        &selected_collection_id,
                                        request,
                                        selected_folder_value_for_worker.unwrap(),
                                    )
                                    .await;
                                    let _ = Gui::refresh_collections(collections_for_worker).await;
                                });
                            }
                        }
                    }
                });
        }
    }
}
