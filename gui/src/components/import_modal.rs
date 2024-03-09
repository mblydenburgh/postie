use api::PostieApi;

use crate::{Gui, ImportMode};

pub fn import_modal(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(mut import_window_open) = gui.import_window_open.try_write() {
        if *import_window_open == true {
            egui::Window::new("Import Modal")
                .open(&mut *import_window_open)
                .show(ctx, |ui| {
                    ui.label("Please copy and paste the file path to import");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut gui.import_file_path);
                        if ui.button("Import").clicked() {
                            let path = gui.import_file_path.to_owned();
                            if let Ok(import_mode) = gui.import_mode.try_read() {
                                let import_result_clone = gui.import_result.clone();
                                match *import_mode {
                                    ImportMode::COLLECTION => {
                                        tokio::spawn(async move {
                                            PostieApi::import_collection(&path).await
                                        });
                                    }
                                    ImportMode::ENVIRONMENT => {
                                        let spawn = tokio::spawn(async move {
                                            let res =
                                                PostieApi::import_environment(&path).await.unwrap();
                                            let mut data = import_result_clone.lock().unwrap();
                                            *data = Some(res);
                                        });
                                    }
                                };
                            }
                        };
                        let i = gui.import_result.lock().unwrap();
                        if let Some(import_res) = &*i {
                            ui.label(import_res);
                        }
                    });
                });
        }
    }
}
