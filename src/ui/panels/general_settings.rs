use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::Color32;

use crate::events::{Reset, UpdateWalls};
use crate::simulation::grid::Grid;
use crate::ui::draw::{EventSystemParams, QuerySystemParams};
use crate::ui::state::*;

pub fn draw_general_settings(
    sets: &mut QuerySystemParams,
    ui_state: &mut UiState,
    ui: &mut egui::Ui,
    events: &mut EventSystemParams,
    commands: &mut Commands,
    grid: &mut Grid,
    diagnostics: Res<DiagnosticsStore>,
    sim_time: &SimTime,
    fixed_timestep: &mut Time<Fixed>,
) {
    egui::TopBottomPanel::bottom("quick_settings_bottom_panel").show_inside(ui, |ui| {
        ui.add_space(3.);
        ui.heading("Quick Settings");
        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .button(if ui_state.is_running { "Stop" } else { "Start" })
                .clicked()
            {
                ui_state.is_running = !ui_state.is_running;
            }

            if ui.button("Reset").clicked() {
                events.reset_ev.send(Reset { force: true });
            }

            if ui
                .add(egui::Button::new("Delete all").fill(Color32::DARK_RED))
                .clicked()
            {
                for (e, _) in sets.source_set.p0().iter() {
                    commands.entity(e).despawn();
                }
                for (e, _) in sets.rect_wall_set.p0().iter() {
                    commands.entity(e).despawn();
                }
                for (e, _) in sets.circ_wall_set.p0().iter() {
                    commands.entity(e).despawn();
                }
                for (e, _) in sets.mic_set.p0().iter() {
                    commands.entity(e).despawn();
                }

                grid.reset_cells(ui_state.boundary_width);
                events.wall_update_ev.send(UpdateWalls);
            }

            ui.checkbox(&mut ui_state.reset_on_change, "Reset on change");

            if ui
                .checkbox(&mut ui_state.show_plots, "Show Plots")
                .clicked()
            {
                for (_, mut mic) in sets.mic_set.p0().iter_mut() {
                    mic.clear();
                }
            }
        });

        ui.add_space(5.);

        ui.horizontal(|ui| {
            if ui
                .add(egui::Slider::new(&mut ui_state.framerate, 1f64..=500.).logarithmic(true))
                .changed()
            {
                if ui_state.read_epilepsy_warning {
                    fixed_timestep.set_timestep_hz(ui_state.framerate);
                } else {
                    ui_state.show_epilepsy_warning = true;
                    ui_state.framerate = 60.;
                }
            }
            ui.add_space(5.);
            ui.label("Simulation Frame Rate");
        });

        ui.add_space(5.);

        ui.horizontal(|ui| {
            ui.label(format!(
                "Simulation Time: {:.5} ms",
                sim_time.time_since_start * 1000.
            ));

            ui.add(egui::Separator::default().vertical());
            ui.label(format!(
                "FPS: {:.1}",
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                    .unwrap_or(0.0)
            ));
        });

        ui.add_space(5.);
    });
}
