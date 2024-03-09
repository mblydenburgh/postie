use egui::SidePanel;

use crate::{ActiveWindow, Gui};

pub fn side_panel(gui: &mut Gui, ctx: &egui::Context) {
    SidePanel::left("nav_panel").show(ctx, |ui| {
        if let Ok(mut active_window) = gui.active_window.try_write() {
            if ui.button("Collections").clicked() {
                *active_window = ActiveWindow::COLLECTIONS;
            }
            if ui.button("Environment").clicked() {
                *active_window = ActiveWindow::ENVIRONMENT;
            }
            if ui.button("History").clicked() {
                *active_window = ActiveWindow::HISTORY;
            }
        }
    });
}
