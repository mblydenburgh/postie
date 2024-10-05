use std::{thread, time};

use api::PostieApi;

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
                                        let blank_collection = api::RequestCollection {
                                            name: gui.new_name.clone(),
                                            requests: vec![],
                                        };
                                        let collections_for_worker = gui.collections.clone();
                                        let _ = tokio::spawn(async move {
                                            let _ = PostieApi::save_collection(blank_collection)
                                                .await
                                                .unwrap();

                                        });
                                        _ = tokio::spawn(async move {
                                            let sleep = time::Duration::from_millis(100);
                                            thread::sleep(sleep);
                                            Gui::refresh_collections(collections_for_worker).await;
                                        });
                                    },
                                    ImportMode::ENVIRONMENT => {
                                    }
                                }
                            }
                        }
                    })
                });
        }
    }
}
