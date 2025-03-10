use egui::{Layout, Vec2};
use egui_extras::{Column, TableBuilder};

use super::draw::CTRL_KEY_TEXT;
use super::state::UiState;

pub fn draw_keybinds(ui_state: &mut UiState, ctx: &egui::Context) {
    egui::Window::new("Keybinds")
        .open(&mut ui_state.show_keybinds)
        .default_size(Vec2::new(400., 400.))
        .resizable(false)
        .collapsible(false)
        .constrain(true)
        .show(ctx, |ui| {
            TableBuilder::new(ui)
                .resizable(false)
                .striped(true)
                .column(Column::remainder())
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Action");
                    });
                    header.col(|ui| {
                        ui.strong("Keybind");
                    });
                })
                .body(|mut body| {
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.strong("File Actions"); //Title case??
                            });
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("New simulation");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+N"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Save simulation");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+S"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Open simulation");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+O"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Quit simulation");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+Q"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.strong("General");
                            });
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Undo");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+Z"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Redo");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+Shift+Z"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Copy selected");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+C"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Paste clipboard");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+V"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.strong("Simulation");
                            });
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Play/Pause");
                        });
                        row.col(|ui| {
                            ui.label("Space");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.strong("Tools");
                            });
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Delete selected");
                        });
                        row.col(|ui| {
                            ui.label("X");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Snap to grid");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT} + \"Edit\""));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Select tool");
                        });
                        row.col(|ui| {
                            ui.label("Q");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Edit tool");
                        });
                        row.col(|ui| {
                            ui.label("E");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place rectangular wall tool");
                        });
                        row.col(|ui| {
                            ui.label("R");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place circular wall tool");
                        });
                        row.col(|ui| {
                            ui.label("C");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place source tool");
                        });
                        row.col(|ui| {
                            ui.label("S");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place microphone tool");
                        });
                        row.col(|ui| {
                            ui.label("M");
                        });
                    });
                });
        });
}
