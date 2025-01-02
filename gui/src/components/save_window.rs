use std::{str::FromStr, sync::Arc};

use api::{
    domain::{
        collection::{CollectionFolder, CollectionItemOrFolder},
        environment::EnvironmentFile,
    },
    HttpRequest, PostieApi, RequestBody,
};
use uuid::Uuid;

use crate::Gui;

pub fn save_window(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(mut save_window_open) = gui.save_window_open.try_write() {
        if *save_window_open == true {
            egui::Window::new("Save request")
                .open(&mut *save_window_open)
                .show(ctx, |ui| {
                    let collections_clone = Arc::clone(&gui.collections);
                    let collections = collections_clone.try_write().unwrap().clone();
                    if let Some(cols) = collections.clone() {
                        let selected_collection = cols.clone().first().unwrap().clone();
                        let mut selected_request_folder =
                            match selected_collection.item.first().unwrap() {
                                CollectionItemOrFolder::Folder(folder) => folder,
                                CollectionItemOrFolder::Item(_) => &CollectionFolder {
                                    name: "Root".to_string(),
                                    item: vec![],
                                },
                            };
                        egui::ComboBox::from_label("Collection to add request to")
                            .selected_text(selected_collection.info.name.clone())
                            .show_ui(ui, |ui| {
                                for col in cols {
                                    ui.selectable_value(
                                        &mut selected_collection.clone(),
                                        col.clone(),
                                        col.clone().info.name,
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
                            println!(
                                "Saving request to collection: {:?}",
                                selected_collection.info.name
                            );
                            println!("Saving to folder: {:?}", selected_request_folder.name);
                            let active_tab = gui.active_tab.try_read().unwrap().clone();
                            if let Some(active_tab) = active_tab.clone() {
                                println!("updating request with tab info: {:?}", active_tab);
                                /* TODO - save a new request to the selected collection using tab
                                info.

                                Get collection, match sub folder, append new item to list
                                */
                                let _ = tokio::spawn(async move {
                                    // call new add_req_to_collection method
                                    let mut headers: Vec<(String, String)> = vec![];
                                    for header in active_tab.req_headers.into_iter() {
                                        headers.push((header.key.clone(), header.value.clone()));
                                    }
                                    let request: HttpRequest = HttpRequest {
                                        tab_id: Uuid::from_str(&active_tab.id.clone()).unwrap(),
                                        id: Uuid::new_v4(),
                                        name: None,
                                        method: active_tab.method.clone(),
                                        url: active_tab.url.clone(),
                                        headers: Some(headers),
                                        body: match serde_json::from_str(
                                            &active_tab.req_body.clone().as_str(),
                                        ) {
                                            Ok(body) => Some(RequestBody::JSON(body)),
                                            Err(_) => None,
                                        },
                                        environment: EnvironmentFile {
                                            id: "".into(),
                                            name: "".into(),
                                            values: None,
                                        },
                                    };
                                    let _ = PostieApi::add_request_to_collection(
                                        &selected_collection.info.id,
                                        &request,
                                    )
                                    .await;
                                });
                            }
                        }
                    }
                });
        }
    }
}
