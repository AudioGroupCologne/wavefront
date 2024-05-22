use egui::Vec2;
use egui_extras::{Column, TableBuilder};

use super::draw::CTRL_KEY_TEXT;
use super::state::UiState;

pub fn draw_help(ui_state: &mut UiState, ctx: &egui::Context) {
    egui::Window::new("Keybinds")
        .open(&mut ui_state.show_help)
        .default_size(Vec2::new(400., 400.))
        .resizable(false)
        .collapsible(false)
        .constrain(true)
        .show(ctx, |ui| {
            // TODO: add links to documentation/user manual

            ui.heading("Keybinds");

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
                            ui.label("Delete Selected");
                        });
                        row.col(|ui| {
                            ui.label("Backspace or Delete");
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
                            ui.label("Copy Selected");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+C"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Paste Clipboard");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+V"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Save Current Simulation");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+S"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Load Previous Simulation");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT}+L"));
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Snap to Grid");
                        });
                        row.col(|ui| {
                            ui.label(format!("{CTRL_KEY_TEXT} + Move or Resize Wall"));
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
                            ui.label("Select Tool");
                        });
                        row.col(|ui| {
                            ui.label("Q");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Move Tool");
                        });
                        row.col(|ui| {
                            ui.label("W");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Resize Tool");
                        });
                        row.col(|ui| {
                            ui.label("E");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place Rectangle Wall Tool");
                        });
                        row.col(|ui| {
                            ui.label("R");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place Circular Wall Tool");
                        });
                        row.col(|ui| {
                            ui.label("C");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place Source Tool");
                        });
                        row.col(|ui| {
                            ui.label("S");
                        });
                    });
                    body.row(15.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Place Microphone Tool");
                        });
                        row.col(|ui| {
                            ui.label("M");
                        });
                    });
                });
        });
}
