use eframe::{
    egui::{CentralPanel, SidePanel},
    epi::App,
    run_native, NativeOptions,
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum ActiveWindow {
    REQUEST,
    ENVIRONMENT
}
#[derive(Serialize, Deserialize)]
pub enum RequestWindowMode {
    PARAMS,
    HEADERS,
    BODY
}

#[derive(Serialize, Deserialize)]
pub struct GuiConfig {
    pub active_window: ActiveWindow,
    pub request_window_mode: RequestWindowMode
}
impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            active_window: ActiveWindow::REQUEST,
            request_window_mode: RequestWindowMode::BODY
        }
    }
}

pub struct Gui {
    pub config: GuiConfig
}
impl Default for Gui {
    fn default() -> Self {
        Self {
            config: GuiConfig::default()
        }
    }
}
impl App for Gui {
    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        let mut url = String::from("");
        SidePanel::left("nav_panel").show(ctx, |ui| {
            if ui.button("Request").clicked() {
                self.config.active_window = ActiveWindow::REQUEST;
            }
            if ui.button("Environment").clicked() {
                self.config.active_window = ActiveWindow::ENVIRONMENT;
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            SidePanel::left("content_panel").show(ctx, |ui| {
                match self.config.active_window {
                    ActiveWindow::REQUEST => {
                        ui.label("Collections");
                    }
                    ActiveWindow::ENVIRONMENT => {
                        ui.label("Environments");
                    }
                }
            });
            ui.heading("Welcome to Postie!");
            ui.horizontal(|ui| {
                ui.label("URL:");
                let url_updated = ui.text_edit_singleline(&mut url);
                if url_updated.changed() {
                    println!("{}", url)
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Params").clicked() {}
                if ui.button("Headers").clicked() {}
                if ui.button("Body").clicked() {}
            });
        });
        match self.config.request_window_mode {
            RequestWindowMode::BODY => {
            }
            RequestWindowMode::PARAMS => {
            }
            RequestWindowMode::HEADERS => {
            }
        }
    }

    fn name(&self) -> &str {
        "Postie"
    }
}

fn main() {
    let app = Gui::default();
    let native_options = NativeOptions::default();
    run_native(Box::new(app), native_options)
}
