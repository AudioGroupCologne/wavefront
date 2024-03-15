use egui::{Painter, Pos2, Rect};

use crate::ui::state::ToolType;

pub trait GizmoComponent {
    fn get_gizmo_positions(&self, tool_type: &ToolType) -> Vec<Pos2>;

    fn draw_gizmo(
        &self,
        painter: &Painter,
        tool_type: &ToolType,
        highlight: bool,
        image_rect: &Rect,
        delta_l: f32,
    );
}
