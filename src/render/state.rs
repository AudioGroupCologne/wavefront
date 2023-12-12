use bevy::prelude::*;
use bevy_pixel_buffer::bevy_egui::egui;

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
    pub show_fft: bool,
    pub show_mic_plot: bool,
    pub current_fft_microphone: Option<usize>,
    pub plot_type: PlotType,
    pub tool_type: ToolType,
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
            show_fft: false,
            show_mic_plot: false,
            current_fft_microphone: None,
            plot_type: PlotType::TimeDomain,
            tool_type: ToolType::PlaceSource,
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
