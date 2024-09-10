use std::fmt;

use bevy::prelude::*;

/// A resource to store the current simulation time in seconds.
#[derive(Default, Resource)]
pub struct SimTime {
    /// Time since simulation start in seconds
    pub time_since_start: f32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    Select,
    Place(PlaceType),
    Move,
    ResizeWall,
}

impl fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::Select => write!(f, "Select Object"),
            ToolType::Place(_) => write!(f, "Place Object"),
            ToolType::Move => write!(f, "Move Object"),
            ToolType::ResizeWall => write!(f, "Resize Wall"),
        }
    }
}

/// The different types of objects that can be placed in the simulation.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlaceType {
    Source,
    Mic,
    RectWall,
    CircWall,
}

impl fmt::Display for PlaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceType::Source => write!(f, "Source"),
            PlaceType::Mic => write!(f, "Microphone"),
            PlaceType::RectWall => write!(f, "Rectangle Wall"),
            PlaceType::CircWall => write!(f, "Circle Wall"),
        }
    }
}

#[derive(Default, Resource)]
pub struct FftMicrophone {
    pub mic_id: Option<usize>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FftScaling {
    Normalized,
    Decibels,
}

impl fmt::Display for FftScaling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FftScaling::Normalized => write!(f, "Normalized"),
            FftScaling::Decibels => write!(f, "dB"),
        }
    }
}

/// A resource to store the current state of the UI.
/// This includes the current tool, the current place type, and various other settings.
#[derive(Resource, PartialEq, Clone, Copy)]
pub struct UiState {
    pub is_running: bool,
    pub delta_l: f32,
    pub boundary_width: u32,
    pub render_abc_area: bool,
    pub image_rect: egui::Rect,
    pub show_plots: bool,
    pub current_tool: ToolType,
    pub cur_place_type: PlaceType,
    pub wall_reflection_factor: f32,
    pub wall_is_hollow: bool,
    pub tools_enabled: bool,
    pub reset_on_change: bool,
    pub tool_use_enabled: bool,
    pub show_preferences: bool,
    pub show_about: bool,
    pub show_help: bool,
    pub fft_scaling: FftScaling,
    pub framerate: f64,
    pub scroll_volume_plot: bool,
    pub highest_y_volume_plot: f64,
    pub show_epilepsy_warning: bool,
    pub read_epilepsy_warning: bool,
    pub show_fft_approx: bool,
    pub fft_window_size: usize,
    pub collapse_header: bool,
    pub max_gradient: f32,
    pub min_gradient: f32,
    pub hide_gizmos: bool,
    pub show_new_warning: bool,
    pub show_frequencies: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_running: cfg!(debug_assertions),
            // set to result in a sample rate of 4800kHz
            delta_l: 0.00715,
            boundary_width: 50,
            render_abc_area: false,
            image_rect: egui::Rect::NOTHING,
            show_plots: false,
            current_tool: ToolType::Place(PlaceType::Source),
            cur_place_type: PlaceType::Source,
            wall_reflection_factor: 1.,
            wall_is_hollow: false,
            tools_enabled: true,
            reset_on_change: true,
            tool_use_enabled: true,
            show_preferences: false,
            show_about: false,
            show_help: false,
            fft_scaling: FftScaling::Normalized,
            framerate: 60.,
            scroll_volume_plot: true,
            highest_y_volume_plot: 0.,
            show_epilepsy_warning: false,
            read_epilepsy_warning: false,
            show_fft_approx: false,
            fft_window_size: 1024,
            collapse_header: false,
            max_gradient: 2.,
            min_gradient: -2.,
            hide_gizmos: false,
            show_new_warning: false,
            show_frequencies: false,
        }
    }
}

/// A resource to store the currently copied [`Entity`] for the clipboard.
#[derive(Resource, Default)]
pub struct ClipboardBuffer {
    buffer: Option<Entity>,
}

impl ClipboardBuffer {
    /// Clears the clipboard buffer.
    pub fn clear(&mut self) {
        self.buffer = None;
    }

    /// Returns the [`Entity`] stored in the clipboard buffer. Returns `None` if the buffer is empty.
    pub fn get(&mut self) -> Option<Entity> {
        self.buffer
    }

    /// Copies an [`Entity`] to the clipboard buffer.
    pub fn copy(&mut self, entity: Entity) {
        self.buffer = Some(entity);
    }
}
