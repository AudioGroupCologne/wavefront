use std::fmt;

use bevy::prelude::*;

/// A resource to store the current simulation time in seconds.
#[derive(Default, Resource)]
pub struct SimTime {
    /// Time since simulation start in seconds
    pub time_since_start: f32,
    /// Samples since simulation start
    pub samples_since_start: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    Select,
    Place(PlaceType),
    Edit,
}

impl fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::Select => write!(f, "Select Object"),
            ToolType::Place(_) => write!(f, "Place Object"),
            ToolType::Edit => write!(f, "Edit Object"),
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
    pub current_tool: ToolType,
    pub cur_place_type: PlaceType,
    pub wall_reflection_factor: f32,
    pub wall_is_hollow: bool,
    pub tools_enabled: bool,
    pub reset_on_change: bool,
    pub tool_use_enabled: bool,
    pub framerate: f64,
    pub scroll_volume_plot: bool,
    pub highest_y_volume_plot: f64,
    pub max_gradient: f32,
    pub min_gradient: f32,
    pub hide_gizmos: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_running: true,
            // set to result in a sample rate of 48kHz
            delta_l: 0.00715,
            boundary_width: 50,
            render_abc_area: false,
            image_rect: egui::Rect::NOTHING,
            current_tool: ToolType::Select,
            cur_place_type: PlaceType::Source,
            wall_reflection_factor: 1.,
            wall_is_hollow: false,
            tools_enabled: true,
            reset_on_change: true,
            tool_use_enabled: true,
            framerate: 30.,
            scroll_volume_plot: true,
            highest_y_volume_plot: 0.,
            max_gradient: 1.25,
            min_gradient: -1.25,
            hide_gizmos: false,
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
