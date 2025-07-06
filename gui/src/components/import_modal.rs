use api::domain::ui;

use crate::Gui;

pub fn import_modal(gui: &mut Gui, ctx: &egui::Context) {
  if let Ok(mut import_window_open) = gui.gui_state.import_window_open.try_write() {
    if *import_window_open {
      egui::Window::new("Import File")
        .open(&mut import_window_open)
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
                gui.gui_state.import_file_path = file.to_str().unwrap_or("").into();
              }
            };
            ui.text_edit_singleline(&mut gui.gui_state.import_file_path);
            if ui.button("Import").clicked() {
              let path = gui.gui_state.import_file_path.to_owned();
              if let Ok(import_mode) = gui.gui_state.import_mode.try_read() {
                let import_result_clone = gui.worker_state.import_result.clone();
                match *import_mode {
                  ui::ImportMode::COLLECTION => {
                    let api_for_worker = std::sync::Arc::clone(&gui.worker_state.api);
                    _ = tokio::spawn(async move {
                      let res = api_for_worker
                        .write()
                        .await
                        .import_collection(&path)
                        .await
                        .unwrap();
                      let mut data = import_result_clone.lock().unwrap();
                      *data = Some(res);
                    });
                    gui
                      .event_tx
                      .send(crate::events::GuiEvent::RefreshCollections());
                  }
                  ui::ImportMode::ENVIRONMENT => {
                    let api_for_worker = std::sync::Arc::clone(&gui.worker_state.api);
                    _ = tokio::spawn(async move {
                      let res = api_for_worker
                        .write()
                        .await
                        .import_environment(&path)
                        .await
                        .unwrap();
                      let mut data = import_result_clone.lock().unwrap();
                      *data = Some(res);
                    });
                    gui
                      .event_tx
                      .send(crate::events::GuiEvent::RefreshEnvironments());
                  }
                };
              }
              ui.close_menu();
            }
            let i = gui.worker_state.import_result.lock().unwrap();
            if let Some(import_res) = &*i {
              ui.label(import_res);
            }
          });
        });
    }
  }
}
