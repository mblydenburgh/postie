use std::cell::RefMut;

use api::{domain::environment::EnvironmentValue, ResponseData};
use egui::{CentralPanel, ScrollArea, TextEdit, TextStyle, TopBottomPanel};
use egui_extras::{Column, TableBuilder};
use egui_json_tree::JsonTree;

use crate::{Gui, RequestWindowMode};

pub fn content_panel(gui: &mut Gui, ctx: &egui::Context) {
    if let Ok(request_window_mode) = gui.request_window_mode.try_read() {
        match *request_window_mode {
            RequestWindowMode::BODY => {
                TopBottomPanel::top("request_panel")
                    .resizable(true)
                    .min_height(250.0)
                    .show(ctx, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(&mut gui.body_str)
                                    .code_editor()
                                    .desired_rows(20)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY)
                                    .font(TextStyle::Monospace),
                            );
                        });
                    });
                if gui.response.try_read().unwrap().is_some() {
                    CentralPanel::default().show(ctx, |ui| {
                        let binding = gui.response.try_read().unwrap();
                        let r = binding.as_ref().unwrap();
                        match r {
                            ResponseData::JSON(json) => {
                                ScrollArea::vertical().show(ui, |ui| {
                                    JsonTree::new("response-json", json).show(ui);
                                });
                            }
                            ResponseData::TEXT(text) => {
                                ScrollArea::vertical().show(ui, |ui| {
                                    ui.label(text);
                                });
                            }
                        };
                    });
                }
            }
            RequestWindowMode::PARAMS => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.label("params");
                });
            }
            RequestWindowMode::HEADERS => {
                CentralPanel::default().show(ctx, |ui| {
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::auto());
                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Enabled");
                            });
                            header.col(|ui| {
                                ui.strong("Key");
                            });
                            header.col(|ui| {
                                ui.strong("Value");
                            });
                        })
                        .body(|mut body| {
                            for header in gui.headers.borrow_mut().iter_mut() {
                                body.row(30.0, |mut row| {
                                    let (enabled, key, value) = header;
                                    row.col(|ui| {
                                        ui.checkbox(enabled, "");
                                    });
                                    row.col(|ui| {
                                        ui.text_edit_singleline(key);
                                    });
                                    row.col(|ui| {
                                        ui.text_edit_singleline(value);
                                    });
                                });
                            }
                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("Add").clicked() {
                                        gui.headers.borrow_mut().push((
                                            true,
                                            String::from(""),
                                            String::from(""),
                                        ));
                                    };
                                });
                            });
                        });
                });
            }
            RequestWindowMode::ENVIRONMENT => {
                CentralPanel::default().show(ctx, |ui| {
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::auto())
                        .column(Column::auto());
                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Enabled");
                            });
                            header.col(|ui| {
                                ui.strong("Key");
                            });
                            header.col(|ui| {
                                ui.strong("Value");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
                        })
                        .body(|mut body| {
                            let selected_environment = gui.selected_environment.borrow_mut();
                            let mut values_ref =
                                RefMut::map(selected_environment, |env| &mut env.values);
                            if let Some(values) = values_ref.as_mut() {
                                for env_var in values {
                                    body.row(30.0, |mut row| {
                                        row.col(|ui| {
                                            ui.checkbox(&mut env_var.enabled, "");
                                        });
                                        row.col(|ui| {
                                            ui.text_edit_singleline(&mut env_var.key);
                                        });
                                        row.col(|ui| {
                                            ui.text_edit_singleline(&mut env_var.value);
                                        });
                                        row.col(|ui| {
                                            ui.text_edit_singleline(&mut env_var.r#type);
                                        });
                                    });
                                }
                            }
                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("Add").clicked() {
                                        if let Some(vals) = values_ref.as_mut() {
                                            vals.push(EnvironmentValue {
                                                key: String::from(""),
                                                value: String::from(""),
                                                r#type: String::from("default"),
                                                enabled: true,
                                            });
                                        }
                                    };
                                });
                            });
                        });
                });
            }
        };
    }
}
