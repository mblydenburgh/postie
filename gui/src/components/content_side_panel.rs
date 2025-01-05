use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};

use crate::Gui;
use api::{
    domain::{
        collection::{CollectionRequest, CollectionUrl},
        request::{DBRequest, HttpMethod},
        response::ResponseData,
        ui,
    },
    PostieApi,
};
use egui::{ScrollArea, SidePanel};

pub fn content_side_panel(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(active_window) = gui.active_window.try_read() {
        SidePanel::left("content_panel").show(ctx, |ui| match *active_window {
                ui::ActiveWindow::COLLECTIONS => {
                    ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Collections");
                    let collections_clone = Arc::clone(&gui.collections);
                    let collections = collections_clone.try_write().unwrap();
                    if let Some(cols) = &*collections {
                        for c in cols {
                            let c_clone = c.clone();
                            ui.horizontal(|ui| {
                                if ui.button("X").clicked() {
                                    let clicked_id = c_clone.info.id;
                                    // call to delete collection by id, refresh collections for ui
                                        let refresh_clone = collections_clone.clone();
                                        tokio::spawn(async move {
                                            let _ = PostieApi::delete_collection(clicked_id).await;
                                            Gui::refresh_collections(refresh_clone).await;
                                        });
                                }
                                ui.collapsing(c_clone.info.name, |ui| {
                                    for i in c_clone.item {
                                        match i {
                                            api::domain::collection::CollectionItemOrFolder::Item(item) => {
                                                if ui.selectable_value(&mut gui.selected_request, Rc::new(RefCell::from(Some(item.clone().request))), item.name.to_string()).clicked() {
                                                    gui.url = item.request.url.raw.clone();
                                                    gui.selected_http_method = HttpMethod::from_str(&item.request.method.clone()).unwrap();
                                                    if let Some(body) = item.request.body {
                                                        if let Some(body_str) = body.raw {
                                                            gui.body_str = body_str;
                                                        }
                                                    }
                                                    if let Some(headers) = item.request.header {
                                                        let constructed_headers: Vec<(bool, String, String)> = headers.into_iter().map(|h| {
                                                            (true, h.key, h.value)
                                                        }).collect();
                                                        gui.headers = Rc::new(RefCell::from(constructed_headers));
                                                    }
                                                }
                                            },
                                            // TODO - figure out how to correctly pass around Gui and
                                            // Ui to be able to call the recursive function. Also
                                            // figure out a way to make the recursive render function
                                            // not return () and always return a Ui::Response. For now,
                                            // handle rendering one level deep of folders. If a folder
                                            // within a folder is found then a dummy request it
                                            // substituted.
                                            api::domain::collection::CollectionItemOrFolder::Folder(folder) => {
                                                ui.horizontal(|ui| {
                                                    if ui.button("X").clicked() {
                                                    }
                                                    if ui.collapsing(folder.name, |ui| {
                                                        for folder_item in folder.item {
                                                            match folder_item {
                                                                api::domain::collection::CollectionItemOrFolder::Item(i) => {
                                                                    ui.horizontal(|ui| {
                                                                        if ui.button("X").clicked() {
                                                                            // get collection, find clicked
                                                                            // item and remove it from
                                                                            // list. then re-update
                                                                            // collections to refresh ui
                                                                            // PostieApi::remove_collection_item()

                                                                        }
                                                                        if ui.selectable_value(&mut gui.selected_request, Rc::new(RefCell::from(Some(i.clone().request))), i.name.to_string())
                                                                            .clicked() {
                                                                                gui.url = i.request.url.raw.clone();
                                                                                gui.selected_http_method = HttpMethod::from_str(&i.request.method.clone()).unwrap();
                                                                                if let Some(body) = i.request.body {
                                                                                    if let Some(body_str) = body.raw {
                                                                                        gui.body_str = body_str;
                                                                                    }
                                                                                }
                                                                                if let Some(headers) = i.request.header {
                                                                                    let constructed_headers: Vec<(bool, String, String)> = headers.into_iter().map(|h| {
                                                                                        (true, h.key, h.value)
                                                                                    }).collect();
                                                                                    gui.headers = Rc::new(RefCell::from(constructed_headers));
                                                                                }
                                                                            }
                                                                    });
                                                                },
                                                                api::domain::collection::CollectionItemOrFolder::Folder(f) => {
                                                                    let fallback_request = CollectionRequest {
                                                                        method: String::from("GET"),
                                                                        url: CollectionUrl {
                                                                            raw: String::from("default"),
                                                                            host: None,
                                                                            path: None,
                                                                        },
                                                                        auth: None,
                                                                        header: None,
                                                                        body: None,
                                                                    };
                                                                    if ui.selectable_value(&mut gui.selected_request, Rc::new(RefCell::from(Some(fallback_request.clone()))), f.name.to_string()).clicked() {
                                                                    }
                                                                },
                                                            }
                                                        }
                                                    }).header_response.clicked() {
                                                    }
                                                });
                                            },
                                        };
                                    }
                                });
                            });
                        }
                    }
                    });
                }
                ui::ActiveWindow::ENVIRONMENT => {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.label("Environments");
                        let envs_clone = Arc::clone(&gui.environments);
                        let envs = envs_clone.try_write().unwrap();
                        if let Some(env_vec) = &*envs {
                            for env in env_vec {
                                ui.selectable_value(
                                    &mut gui.selected_environment,
                                    Rc::new(RefCell::from(env.clone())),
                                    env.name.to_string(),
                                );
                            }
                        }
                    });
                }
                ui::ActiveWindow::HISTORY => {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.label("History");
                        let history_items_clone = Arc::clone(&gui.request_history_items);
                        let history_items = history_items_clone.try_write().unwrap();
                        let request_clone = gui.saved_requests.try_write().unwrap();
                        if let Some(item_vec) = &*history_items {
                            for item in item_vec {
                                let history_reqs = request_clone.as_ref().unwrap();
                                let id = &item.clone().request_id;
                                let req_name = history_reqs.get(id).unwrap_or(&DBRequest {
                                    id: id.clone(),
                                    method: "GET".into(),
                                    url: "n/a".into(),
                                    name: None,
                                    headers: vec![],
                                    body: None
                                }).url.clone();
                                if ui
                                    .selectable_value(
                                        &mut gui.selected_history_item,
                                        Rc::new(RefCell::from(Some(item.clone()))),
                                        format!("{:?}", req_name), // TODO - create function to get name
                                    )
                                        .clicked()
                                {
                                    // TODO - replace url, method, request body, response body
                                    let responses_clone = gui.saved_responses.try_write().unwrap();
                                    let requests = request_clone.as_ref().unwrap();
                                    let responses = responses_clone.as_ref().unwrap();
                                    let historical_request = requests.get(&item.request_id).unwrap();
                                    let historical_response = responses.get(&item.response_id).unwrap();
                                    gui.url = historical_request.url.clone();
                                    gui.selected_http_method =
                                        HttpMethod::from_str(&historical_request.method).unwrap();
                                    match &historical_request.body {
                                        Some(body_json) => {
                                            gui.body_str = body_json.to_string();
                                        }
                                        None => gui.body_str = String::from(""),
                                    }
                                    let ui_response_clone = gui.response.clone();
                                    let mut ui_response_guard = ui_response_clone.try_write().unwrap();
                                    let response_body = &historical_response.body;
                                    match response_body {
                                        Some(body) => {
                                            let json_val = serde_json::json!(&body);
                                            println!("val: {}", json_val);
                                            let parsed_body = match serde_json::from_str(body) {
                                                Ok(b) => ResponseData::JSON(b),
                                                Err(e) => {
                                                    println!("{}", e);
                                                    ResponseData::TEXT(body.clone())
                                                },
                                            };
                                            *ui_response_guard = Some(parsed_body)
                                        },
                                        None => *ui_response_guard = None,
                                    }
                                }
                            }
                        }
                    });
                }
            });
    }
}
