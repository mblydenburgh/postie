use egui::TopBottomPanel;

use crate::{Gui, ImportMode};

pub fn menu_panel(gui: &mut Gui, ctx: &egui::Context) {
    TopBottomPanel::top("menu_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("Menu", |ui| {
                ui.menu_button("New", |ui| {
                    if ui.button("Collection").clicked() {
                        ui.close_menu();
                    };
                    if ui.button("Evnironment").clicked() {
                        ui.close_menu();
                    };
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
        });
    });
}
