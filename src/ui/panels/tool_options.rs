use crate::ui::state::*;

pub fn draw_tool_options(ui_state: &mut UiState, ui: &mut egui::Ui) {
    egui::TopBottomPanel::bottom("tool_options_panel").show_inside(ui, |ui| {
        ui.add_space(3.);
        ui.heading("Tool Options");
        ui.separator();

        ui.set_enabled(!ui_state.render_abc_area);

        match ui_state.current_tool {
            ToolType::Place(_) => {
                egui::ComboBox::from_label("Select Object to Place")
                    .selected_text(format!("{}", ui_state.cur_place_type))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.selectable_value(
                            &mut ui_state.cur_place_type,
                            PlaceType::Source,
                            "Source",
                        );
                        ui.selectable_value(
                            &mut ui_state.cur_place_type,
                            PlaceType::Mic,
                            "Microphone",
                        );
                        ui.selectable_value(
                            &mut ui_state.cur_place_type,
                            PlaceType::RectWall,
                            "Rectangle Wall",
                        );
                        ui.selectable_value(
                            &mut ui_state.cur_place_type,
                            PlaceType::CircWall,
                            "Circular Wall",
                        );
                    });

                if matches!(
                    ui_state.cur_place_type,
                    PlaceType::RectWall | PlaceType::CircWall
                ) {
                    ui.add(
                        egui::Slider::new(&mut ui_state.wall_reflection_factor, 0.0..=1.0)
                            .text("Wall Reflection Factor"),
                    );
                    ui.checkbox(&mut ui_state.wall_is_hollow, "Hollow");
                }
                ui_state.current_tool = ToolType::Place(ui_state.cur_place_type);
            }
            _ => {
                ui.add_space(10.);
                ui.vertical_centered(|ui| ui.label("Select another tool to see its options"));
            }
        }

        ui.add_space(10.);
    });
}
