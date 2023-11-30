use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::EguiContexts;
use bevy_pixel_buffer::{bevy_egui::egui, prelude::*};

use crate::components::{GradientResource, Wall};
use crate::constants::*;
use crate::grid::Grid;

#[derive(Resource)]
pub struct UiState {
    pub value: f32,
    pub delta_l: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            value: 10000.0,
            delta_l: 0.001,
        }
    }
}

pub fn draw_pixels(
    mut pb: QueryPixelBuffer,
    mut egui_context: EguiContexts,
    grid: Res<Grid>,
    gradient: Res<GradientResource>,
    walls: Query<&Wall>,
    mut ui_state: ResMut<UiState>,
) {
    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = grid.cells[Grid::coords_to_index(coords.x + E_AL, coords.y + E_AL, 8)];
        let color = gradient.0.at((p) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });
    // Walls
    for wall in walls.iter() {
        let (x, y) = Grid::index_to_coords(wall.0 as u32);
        //TODO: handle result
        let _ = frame.set(
            UVec2::new(x, y),
            Pixel {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );
    }

    let ctx = egui_context.ctx_mut();
    egui::SidePanel::left("left_panel").show(ctx, |ui| {
        ui.heading("Settings");
        ui.separator();
        ui.label("TODO");

        ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=20000.0).text("value"));
        ui.add(
            egui::Slider::new(&mut ui_state.delta_l, 0.0..=10.0)
                .text("Delta L")
                .logarithmic(true),
        );
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        // pb.update_fill_egui(ui.available_size());

        let texture = pb.egui_texture();
        ui.image(egui::load::SizedTexture::new(texture.id, texture.size));
    });
}
