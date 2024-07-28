use std::{thread, time};

use api::PostieApi;

use crate::{Gui, ImportMode};

pub fn import_modal(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(mut import_window_open) = gui.import_window_open.try_write() {
        if *import_window_open == true {
            egui::Window::new("Import File")
                .open(&mut *import_window_open)
                .show(ctx, |ui| {
                    ui.label("Please select a file or enter path to import");
                    ui.horizontal(|ui| {
                        if ui.button("Browse").clicked() {
                            let file = rfd::FileDialog::new()
                                .add_filter("json", &["json"])
                                .set_directory("/")
                                .pick_file();
                            if let Some(file) = file {
                                println!("Importing file: {:?}", file);
                                gui.import_file_path = file.to_str().unwrap_or("".into()).into();
                            }
                        };
                        ui.text_edit_singleline(&mut gui.import_file_path);
                        if ui.button("Import").clicked() {
                            let path = gui.import_file_path.to_owned();
                            if let Ok(import_mode) = gui.import_mode.try_read() {
                                let import_result_clone = gui.import_result.clone();
                                match *import_mode {
                                    ImportMode::COLLECTION => {
                                        let collections_for_worker = gui.collections.clone();
                                        _ = tokio::spawn(async move {
                                            let res =
                                                PostieApi::import_collection(&path).await.unwrap();
                                            let mut data = import_result_clone.lock().unwrap();
                                            *data = Some(res);
                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(100);
                                            thread::sleep(sleep);
                                            Gui::refresh_collections(collections_for_worker).await;
                                        });
                                    }
                                    ImportMode::ENVIRONMENT => {
                                        let environments_for_worker = gui.environments.clone();
                                        _ = tokio::spawn(async move {
                                            let res =
                                                PostieApi::import_environment(&path).await.unwrap();
                                            let mut data = import_result_clone.lock().unwrap();
                                            *data = Some(res);
                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(100);
                                            thread::sleep(sleep);
                                            Gui::refresh_environments(environments_for_worker)
                                                .await;
                                        });
                                    }
                                };
                            }
                        }
                        let i = gui.import_result.lock().unwrap();
                        if let Some(import_res) = &*i {
                            ui.label(import_res);
                        }
                    });
                });
        }
    }
}
