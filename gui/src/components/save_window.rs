use std::sync::Arc;

use api::domain::{
  collection::CollectionItemOrFolder,
  environment::EnvironmentFile,
  request::{HttpRequest, RequestBody},
};
use uuid::Uuid;

use crate::Gui;

pub fn save_window(gui: &mut Gui, ctx: &egui::Context) {
  let save_window_mode = gui.gui_state.save_window_open.clone();
  if let Ok(mut save_window_open) = save_window_mode.try_write() {
    if *save_window_open {
      egui::Window::new("Save request")
        .open(&mut save_window_open)
        .show(ctx, |ui| {
          let collections = &gui.worker_state.collections.try_read().unwrap().clone();

          // Use struct fields directly
          let selected_collection = &mut gui.gui_state.selected_save_window_collection; // This should be Option<Collection>
          let selected_folder = &mut gui.gui_state.selected_save_window_folder; // This should be Option<String>

          egui::ComboBox::from_label("Collection to add request to")
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

            let active_tab = gui.worker_state.active_tab.try_read().unwrap().clone();
            println!("updating request with tab info: {:?}", active_tab);

            // Use a copy of the needed variables and move them into the async block
            let selected_collection_id = selected_collection.clone().unwrap().info.id.clone();

            let api_for_worker = Arc::clone(&gui.worker_state.api);
            let tab_for_worker = Arc::clone(&gui.worker_state.active_tab);
            let folder_name = selected_folder.clone();
            let tx_clone = gui.event_tx.clone();

            tokio::spawn(async move {
              let tab = tab_for_worker.read().await;
              let headers: Vec<(String, String)> = tab
                .req_headers
                .into_iter()
                .map(|header| (header.key.clone(), header.value.clone()))
                .collect();

              let request: HttpRequest = HttpRequest {
                tab_id: tab.id,
                id: Uuid::new_v4(),
                name: None,
                method: tab.method.clone(),
                url: tab.url.clone(),
                headers: Some(headers),
                body: serde_json::from_str(&tab.req_body)
                  .ok()
                  .map(RequestBody::JSON),
                environment: EnvironmentFile {
                  id: "".into(),
                  name: "".into(),
                  values: None,
                },
              };

              let _ = api_for_worker
                .write()
                .await
                .add_request_to_collection(&selected_collection_id, request, folder_name.unwrap())
                .await;
              tx_clone.send(crate::events::GuiEvent::RefreshCollections());
            });
          }
        });
    }
  };
}
