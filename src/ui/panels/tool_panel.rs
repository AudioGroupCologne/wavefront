use bevy_pixel_buffer::bevy_egui::egui::Color32;
use egui::{Context, Frame, ImageSource, Margin, Vec2};

use crate::ui::state::*;

const IMAGES: [ImageSource; 4] = [
    egui::include_image!("../../../assets/select.png"),
    egui::include_image!("../../../assets/place.png"),
    egui::include_image!("../../../assets/move.png"),
    egui::include_image!("../../../assets/resize_wall.png"),
];

pub fn draw_tool_panel(ui_state: &mut UiState, ctx: &Context) {
    egui::SidePanel::left("tool_panel")
        .frame(
            Frame::default()
                .inner_margin(Margin {
                    left: 8., //looks better
                    right: 10.,
                    top: 10.,
                    bottom: 10.,
                })
                .fill(Color32::from_rgb(25, 25, 25)),
        )
        .default_width(35.)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_enabled(ui_state.tools_enabled);
            let select_icon = &IMAGES[0];
            let place_icon = &IMAGES[1];
            let move_icon = &IMAGES[2];
            let resize_wall_icon = &IMAGES[3];

            if ui
                .add(
                    egui::Button::image(
                        egui::Image::new(select_icon.clone())
                            .fit_to_exact_size(Vec2::new(24., 24.)),
                    )
                    .fill(if matches!(ui_state.current_tool, ToolType::Select) {
                        Color32::DARK_GRAY
                    } else {
                        Color32::TRANSPARENT
                    })
                    .min_size(Vec2::new(0., 35.)),
                )
                .on_hover_text(format!("{}", ToolType::Select))
                .clicked()
            {
                ui_state.current_tool = ToolType::Select;
            }
            ui.add_space(4.);

            if ui
                .add(
                    egui::Button::image(
                        // TODO: change image depending on cur_place_type??
                        egui::Image::new(place_icon.clone()).fit_to_exact_size(Vec2::new(24., 24.)),
                    )
                    .fill(if matches!(ui_state.current_tool, ToolType::Place(..)) {
                        Color32::DARK_GRAY
                    } else {
                        Color32::TRANSPARENT
                    })
                    .min_size(Vec2::new(0., 35.)),
                )
                .on_hover_text(format!("{}", ToolType::Place(PlaceType::Source)))
                .clicked()
            {
                ui_state.current_tool = ToolType::Place(ui_state.cur_place_type);
            }
            ui.add_space(4.);

            if ui
                .add(
                    egui::Button::image(
                        egui::Image::new(move_icon.clone()).fit_to_exact_size(Vec2::new(24., 24.)),
                    )
                    .fill(if matches!(ui_state.current_tool, ToolType::Move) {
                        Color32::DARK_GRAY
                    } else {
                        Color32::TRANSPARENT
                    })
                    .min_size(Vec2::new(0., 35.)),
                )
                .on_hover_text(format!("{}", ToolType::Move))
                .clicked()
            {
                ui_state.current_tool = ToolType::Move;
            }
            ui.add_space(4.);

            if ui
                .add(
                    egui::Button::image(
                        egui::Image::new(resize_wall_icon.clone())
                            .fit_to_exact_size(Vec2::new(24., 24.)),
                    )
                    .fill(if matches!(ui_state.current_tool, ToolType::ResizeWall) {
                        Color32::DARK_GRAY
                    } else {
                        Color32::TRANSPARENT
                    })
                    .min_size(Vec2::new(0., 35.)),
                )
                .on_hover_text(format!("{}", ToolType::ResizeWall))
                .clicked()
            {
                ui_state.current_tool = ToolType::ResizeWall;
            }
        });
}
