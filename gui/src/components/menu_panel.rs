use std::{ops::Deref, str::FromStr, sync::Arc};

use egui::TopBottomPanel;
use uuid::Uuid;

use crate::{Gui, ImportMode, Tab};

pub fn menu_panel(gui: &mut Gui, ctx: &egui::Context) {
    TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("Menu", |ui| {
                ui.menu_button("New", |ui| {
                    if ui.button("Request").clicked() {
                        let mut tabs = gui.tabs.try_write().unwrap();
                        let new_tab = Tab::default();
                        tabs.insert(new_tab.id.clone(), new_tab);
                    }
                    if ui.button("Collection").clicked() {
                        if let Ok(mut new_model_open) = gui.new_window_open.try_write() {
                            *new_model_open = true;
                        }
                        if let Ok(mut new_model_mode) = gui.new_window_mode.try_write() {
                            *new_model_mode = ImportMode::COLLECTION;
                        }
                    };
                    if ui.button("Environment").clicked() {
                        if let Ok(mut new_model_open) = gui.new_window_open.try_write() {
                            *new_model_open = true;
                        }
                        if let Ok(mut new_model_mode) = gui.new_window_mode.try_write() {
                            *new_model_mode = ImportMode::ENVIRONMENT;
                        }
                    };
                });
                ui.menu_button("Save", |ui| {
                    if ui.button("Request").clicked() {
                        if let Ok(mut save_window_open) = gui.save_window_open.try_write() {
                            *save_window_open = true;
                        }
                    }
                });
                ui.menu_button("Import", |ui| {
                    if ui.button("Collection").clicked() {
                        if let Ok(mut import_open) = gui.import_window_open.try_write() {
                            *import_open = true;
                        }
                        if let Ok(mut import_mode) = gui.import_mode.try_write() {
                            *import_mode = ImportMode::COLLECTION;
                        }
                    };
                    if ui.button("Environment").clicked() {
                        if let Ok(mut import_open) = gui.import_window_open.try_write() {
                            *import_open = true;
                        }
                        if let Ok(mut import_mode) = gui.import_mode.try_write() {
                            *import_mode = ImportMode::ENVIRONMENT;
                        }
                    };
                });
                ui.menu_button("Export", |ui| {
                    if ui.button("Collection").clicked() {
                        ui.close_menu();
                    };
                    if ui.button("Environment").clicked() {
                        ui.close_menu();
                    };
                });
            });
            let is_requesting_lock = gui.is_requesting.try_read();
            if is_requesting_lock.is_ok() {
                if let Ok(is_requesting) = is_requesting_lock {
                    match is_requesting.deref() {
                        Some(r) => {
                            if *r {
                                ui.label("Requesting...");
                            } else {
                                let response_status_lock = gui.res_status.try_read();
                                if response_status_lock.is_ok() {
                                    if let Ok(response_status) = response_status_lock {
                                        ui.label(response_status.deref());
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        });
    });
    TopBottomPanel::top("tabs panel").show(ctx, |ui| {
        let tabs_clone = Arc::clone(&gui.tabs);
        let tabs = tabs_clone.try_read().unwrap();
        ui.horizontal(|ui| {
            for tab in &*tabs {
                let name = if tab.1.url == "" {
                    "Unsent Request".to_string()
                } else {
                    tab.1.url.clone()
                };
                ui.horizontal(|ui| {
                    if ui.button("X").clicked() {
                        let id = Uuid::from_str(&tab.1.id).unwrap();
                        gui.spawn_delete_tab(id);
                    }
                    if ui.button(&name).clicked() {
                        gui.set_active_tab(&tab.1.id.clone());
                        gui.set_gui_values_from_active_tab();
                        gui.url = tab.1.url.clone();
                        gui.selected_http_method = tab.1.method.clone();
                        gui.body_str = tab.1.res_body.clone();
                    }
                });
            }
        });
    });
}
