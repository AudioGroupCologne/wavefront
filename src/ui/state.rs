use std::fmt;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct SimTime {
    pub time_since_start: f32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    MoveSource,
    MoveWall,
    MoveMic,
    ResizeWall,

    Place(PlaceType),
}

impl fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::MoveSource => write!(f, "Move Source"),
            ToolType::ResizeWall => write!(f, "Resize Wall"),
            ToolType::MoveWall => write!(f, "Move Wall"),
            ToolType::MoveMic => write!(f, "Move Microphone"),
            ToolType::Place(_) => write!(f, "Place Object"),
        }
    }
}

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

#[derive(Resource, PartialEq, Clone, Copy)]
pub struct UiState {
    pub is_running: bool,
    pub delta_l: f32,
    pub boundary_width: u32,
    pub render_abc_area: bool,
    pub image_rect: egui::Rect,
    pub show_plots: bool,
    pub current_tool: ToolType,
    pub wall_reflection_factor: f32,
    pub wall_is_hollow: bool,
    pub tools_enabled: bool,
    pub reset_on_change: bool,
    pub tool_use_enabled: bool,
    pub gradient_contrast: f32,
    pub show_preferences: bool,
    pub show_about: bool,
    pub show_help: bool,
    pub enable_spectrogram: bool,
    pub fft_scaling: FftScaling,
    pub framerate: f64,
    pub scroll_volume_plot: bool,
    pub highest_y_volume_plot: f64,
    pub show_epilepsy_warning: bool,
    pub read_epilepsy_warning: bool,
    pub show_fft_approx: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_running: cfg!(debug_assertions),
            delta_l: 0.001,
            boundary_width: 50,
            render_abc_area: false,
            image_rect: egui::Rect::NOTHING,
            show_plots: false,
            current_tool: ToolType::Place(PlaceType::Mic),
            wall_reflection_factor: 1.,
            wall_is_hollow: false,
            tools_enabled: true,
            reset_on_change: true,
            tool_use_enabled: true,
            gradient_contrast: 5.,
            show_preferences: false,
            show_about: false,
            show_help: false,
            enable_spectrogram: false,
            fft_scaling: FftScaling::Normalized,
            framerate: 60.,
            scroll_volume_plot: true,
            highest_y_volume_plot: 0.,
            show_epilepsy_warning: false,
            read_epilepsy_warning: false,
            show_fft_approx: false,
        }
    }
}

/// A resource to store the currently copied [`Entity`] for the clipboard.
#[derive(Resource, Default)]
pub struct ClipboardBuffer {
    buffer: Option<Entity>,
}

impl ClipboardBuffer {
    pub fn clear(&mut self) {
        self.buffer = None;
    }

    pub fn get(&mut self) -> Option<Entity> {
        self.buffer
    }

    pub fn copy(&mut self, entity: Entity) {
        self.buffer = Some(entity);
    }
}
