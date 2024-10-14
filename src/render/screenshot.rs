use bevy::ecs::system::Commands;
use bevy_file_dialog::FileDialogExt;

use super::gradient::Gradient;
use crate::math::constants::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::math::transformations::coords_to_index;
use crate::simulation::grid::Grid;
use crate::ui::loading::SceneSaveFileContents;
use crate::ui::state::UiState;

pub fn screenshot_grid(
    ui_state: &UiState,
    grid: &Grid,
    gradient: &Gradient,
    commands: &mut Commands,
) {
    let mut pixels: Vec<u8> = Vec::new();

    for y in ui_state.boundary_width..(SIMULATION_WIDTH + ui_state.boundary_width) {
        for x in ui_state.boundary_width..(SIMULATION_HEIGHT + ui_state.boundary_width) {
            let current_index = coords_to_index(x, y, ui_state.boundary_width);
            if grid.wall_cache[current_index].is_wall {
                let mut reflection_factor = grid.wall_cache[current_index].reflection_factor;
                if reflection_factor == 0. {
                    reflection_factor = 1.;
                }
                pixels.push((reflection_factor * 255.) as u8);
                pixels.push((reflection_factor * 255.) as u8);
                pixels.push((reflection_factor * 255.) as u8);
            } else {
                let pressure = grid.pressure[current_index];

                let [r, g, b] = gradient.at(pressure, ui_state.min_gradient, ui_state.max_gradient);

                // inverse gamma correction to match the brightness/contrast of the simulation
                pixels.push(((r as f32 / 255.).powf(1. / 2.2) * 255.) as u8);
                pixels.push(((g as f32 / 255.).powf(1. / 2.2) * 255.) as u8);
                pixels.push(((b as f32 / 255.).powf(1. / 2.2) * 255.) as u8);
            }
        }
    }

    let mut data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut data);

    let image = image::RgbImage::from_raw(SIMULATION_WIDTH, SIMULATION_HEIGHT, pixels)
        .expect("could not create image");

    image
        .write_with_encoder(encoder)
        .expect("could not write image");

    commands
        .dialog()
        .add_filter("PNG", &["png"])
        .set_file_name("screenshot.png")
        .set_directory("./")
        .set_title("Select a file to save to")
        .save_file::<SceneSaveFileContents>(data);
}
