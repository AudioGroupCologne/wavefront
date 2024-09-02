use bevy::math::UVec2;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;
use bevy_pixel_buffer::query::QueryPixelBuffer;
use egui::{Layout, Vec2};
use egui_extras::{Column, TableBuilder};

use super::draw::EventSystemParams;
use super::state::UiState;
use crate::events::Reset;
use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;

pub fn draw_preferences(
    show_preferences: &mut bool,
    ctx: &egui::Context,
    ui_state_tmp: &mut UiState,
    events: &mut EventSystemParams,
    grid: &mut Grid,
    pixel_buffers: &mut QueryPixelBuffer,
    gradient: &mut Gradient,
) {
    egui::Window::new("Preferences")
            .open(show_preferences)
            .default_size(Vec2::new(400., 400.))
            .resizable(false)
            .collapsible(false)
            .constrain(true)
            .show(ctx, |ui| {
                // ui.set_min_width(800.);
                let row_height = 20f32;

                ui.columns(1, |columns| {
                    columns[0].vertical_centered(|ui| {
                        ui.add_space(5.);
                        ui.heading("General Settings");

                        ui.push_id("general_settings_table", |ui| {
                            TableBuilder::new(ui)
                            .resizable(false)
                            .striped(false)
                            .column(Column::remainder())
                            .column(Column::remainder())
                            .body(|mut body| {
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .add(egui::Slider::new(&mut ui_state_tmp.delta_l, 0.0001..=10.0).logarithmic(true))
                                                .on_hover_text("Change the size of one cell in the simulation in meters.")
                                                .changed()
                                            {
                                                events.reset_ev.send(Reset::default());
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui|{
                                            ui.label("Delta L (m)");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.strong("Gradient");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            egui::ComboBox::from_id_source("gradient_select")
                                                .selected_text(format!("{:?}", gradient))
                                                .show_ui(ui, |ui| {
                                                    let mut g = *gradient;
                                                    ui.selectable_value(&mut g, Gradient::Turbo, "Turbo");
                                                    ui.selectable_value(&mut g, Gradient::Viridis, "Viridis");
                                                    ui.selectable_value(&mut g, Gradient::Magma, "Magma");
                                                    ui.selectable_value(&mut g, Gradient::Inferno, "Inferno");
                                                    ui.selectable_value(&mut g, Gradient::Plasma, "Plasma");
                                                    ui.selectable_value(&mut g, Gradient::Bw, "Bw");
                                                    *gradient = g;
                                                });
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui|{
                                            ui.label("Colormap");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add(
                                                egui::DragValue::new(&mut ui_state_tmp.min_gradient).speed(0.01)
                                            );
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label("Min Gradient");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add(
                                                egui::DragValue::new(&mut ui_state_tmp.max_gradient).speed(0.01)
                                            );
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                                            ui.label("Max Gradient");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.strong("Boundary");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .checkbox(&mut ui_state_tmp.render_abc_area, "")
                                                .clicked()
                                            {
                                                ui_state_tmp.tools_enabled = !ui_state_tmp.render_abc_area;
                                                let mut pb = pixel_buffers.iter_mut().next().expect("one pixel buffer");

                                                pb.pixel_buffer.size = PixelBufferSize {
                                                    size: if ui_state_tmp.render_abc_area {
                                                        UVec2::new(
                                                            SIMULATION_WIDTH + 2 * ui_state_tmp.boundary_width,
                                                            SIMULATION_HEIGHT + 2 * ui_state_tmp.boundary_width,
                                                        )
                                                    } else {
                                                        UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)
                                                    },
                                                    pixel_size: UVec2::new(1, 1),
                                                };
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui|{
                                            ui.label("Show absorbing boundary");
                                        });
                                    });
                                });
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .add(
                                                    egui::Slider::new(&mut ui_state_tmp.boundary_width, 2..=200),
                                                )
                                                .on_hover_text("Change the width of the boundary. (higher values lead to slower simulation)")
                                                .changed()
                                            {
                                                grid.reset_cells(ui_state_tmp.boundary_width);
                                                grid.reset_walls(ui_state_tmp.boundary_width);
                                                grid.cache_boundaries(ui_state_tmp.boundary_width);
                                                let mut pb = pixel_buffers.iter_mut().next().expect("one pixel buffer");
                                                pb.pixel_buffer.size = PixelBufferSize {
                                                    size: if ui_state_tmp.render_abc_area {
                                                        UVec2::new(
                                                            SIMULATION_WIDTH + 2 * ui_state_tmp.boundary_width,
                                                            SIMULATION_HEIGHT + 2 * ui_state_tmp.boundary_width,
                                                        )
                                                    } else {
                                                        UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)
                                                    },
                                                    pixel_size: UVec2::new(1, 1),
                                                };
                                            }
                                        });
                                    });
                                    row.col(|ui| {
                                        ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui|{
                                            ui.label("Boundary width (px)");
                                        });
                                    });
                                });
                            });
                        });

                        ui.add_space(5.);
                        ui.separator();
                        ui.add_space(5.);

                        ui.heading("Experimental Settings");

                        ui.push_id("experimental_settings_table", |ui| {
                            TableBuilder::new(ui)
                                .resizable(false)
                                .striped(false)
                                .column(Column::remainder())
                                .column(Column::remainder())
                                .body(|mut body| {
                                    body.row(row_height, |mut row| {
                                        row.col(|ui| {
                                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.checkbox(&mut ui_state_tmp.enable_spectrogram, "");
                                            });
                                        });
                                        row.col(|ui| {
                                            ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui|{
                                                ui.label("Spectrogram enabled");
                                            });
                                        });
                                    });
                                });
                            });

                            ui.add_space(5.);

                        });
                });
            });
}
