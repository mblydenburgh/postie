use std::{thread, time};

use api::PostieApi;
use uuid::Uuid;

use crate::{Gui, ImportMode};

pub fn new_modal(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(mut new_window_open) = gui.new_window_open.try_write() {
        if *new_window_open == true {
            egui::Window::new("Create new")
                .open(&mut *new_window_open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Enter name: ");
                        ui.text_edit_singleline(&mut gui.new_name);
                        if ui.button("Save").clicked() {
                            if let Ok(new_window_mode) = gui.new_window_mode.try_read() {
                                match *new_window_mode {
                                    ImportMode::COLLECTION => {
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
                                            let _ = PostieApi::save_collection(blank_collection)
                                                .await
                                                .unwrap();
                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(50);
                                            thread::sleep(sleep);
                                            Gui::refresh_collections(collections_for_worker).await;
                                        });
                                    }
                                    ImportMode::ENVIRONMENT => {
                                        let blank_env = api::domain::environment::EnvironmentFile {
                                            id: Uuid::new_v4().to_string(),
                                            name: gui.new_name.clone(),
                                            values: None,
                                        };
                                        let envs_for_worker = gui.environments.clone();
                                        let _ = tokio::spawn(async move {
                                            let _ = PostieApi::save_environment(blank_env)
                                                .await
                                                .unwrap();
                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(50);
                                            thread::sleep(sleep);
                                            Gui::refresh_environments(envs_for_worker).await;
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
