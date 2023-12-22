use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui::{self, Vec2};

#[derive(Default, Resource)]
pub struct GameTicks {
    pub ticks_since_start: u64,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AttenuationType {
    Power,
    OriginalOneWay,
    Linear,
    Old,
    DoNothing,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlotType {
    TimeDomain,
    FrequencyDomain,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    PlaceSource,
    MoveSource,
    DrawWall,
    MoveWall,
    ResizeWall,
}

impl ToolType {
    pub const TYPES: [Self; 5] = [
        ToolType::MoveSource,
        ToolType::PlaceSource,
        ToolType::DrawWall,
        ToolType::MoveWall,
        ToolType::ResizeWall,
    ]; //not very pretty but works for now
}

#[derive(Debug, PartialEq)]
pub enum WallBrush {
    /// Circular brush
    CircleBrush,
    Rectangle,
}

#[derive(Resource)]
pub struct UiState {
    pub is_running: bool,
    pub delta_l: f32,
    pub epsilon: f32,
    pub e_al: u32,
    pub render_abc_area: bool,
    pub at_type: AttenuationType,
    pub power_order: u32,
    pub image_rect: egui::emath::Rect,
    pub show_plots: bool,
    pub current_fft_microphone: Option<usize>,
    pub spectrum_size: Vec2,
    pub plot_type: PlotType,
    pub current_tool: ToolType,
    pub wall_reflection_factor: f32,
    pub wall_brush: WallBrush,
    pub wall_brush_radius: u32,
    pub tools_enabled: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_running: true,
            delta_l: 0.001,
            epsilon: 0.001,
            e_al: 50,
            render_abc_area: false,
            at_type: AttenuationType::Power,
            power_order: 5,
            image_rect: egui::emath::Rect::NOTHING,
            show_plots: false,
            current_fft_microphone: None,
            spectrum_size: Vec2 { x: 250., y: 500. }, // random init values
            plot_type: PlotType::TimeDomain,
            current_tool: ToolType::MoveSource,
            wall_reflection_factor: 1.,
            wall_brush: WallBrush::Rectangle,
            wall_brush_radius: 10,
            tools_enabled: true,
        }
    }
}

pub struct Images {
    pub cursor_icon: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            cursor_icon: asset_server.load("test_icon.png"),
        }
    }
}
