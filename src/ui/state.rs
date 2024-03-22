use std::fmt;

use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WallType {
    Rectangle,
    Circle,
}

#[derive(Default, Resource)]
pub struct GameTicks {
    pub ticks_since_start: u64,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    PlaceSource,
    MoveSource,
    DrawWall,
    ResizeWall,
    MoveWall,
    PlaceMic,
    MoveMic,
}

impl fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::PlaceSource => write!(f, "Place Source"),
            ToolType::MoveSource => write!(f, "Move Source"),
            ToolType::DrawWall => write!(f, "Draw Wall"),
            ToolType::ResizeWall => write!(f, "Resize Wall"),
            ToolType::MoveWall => write!(f, "Move Wall"),
            ToolType::PlaceMic => write!(f, "Place Mic"),
            ToolType::MoveMic => write!(f, "Move Mic"),
        }
    }
}

#[derive(Default, Resource)]
pub struct FftMicrophone {
    pub mic_id: Option<usize>,
}

#[derive(Resource, PartialEq, Clone, Copy)]
pub struct UiState {
    pub is_running: bool,
    pub delta_l: f32,
    pub boundary_width: u32,
    pub render_abc_area: bool,
    pub power_order: u32,
    pub image_rect: egui::Rect,
    pub show_plots: bool,
    pub current_tool: ToolType,
    pub wall_reflection_factor: f32,
    pub wall_type: WallType,
    pub wall_is_hollow: bool,
    pub tools_enabled: bool,
    pub reset_on_change: bool,
    pub tool_use_enabled: bool,
    pub gradient_contrast: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            is_running: true,
            delta_l: 0.001,
            boundary_width: 50,
            render_abc_area: false,
            power_order: 5,
            image_rect: egui::Rect::NOTHING,
            show_plots: false,
            current_tool: ToolType::MoveSource,
            wall_reflection_factor: 1.,
            wall_type: WallType::Rectangle,
            wall_is_hollow: false,
            tools_enabled: true,
            reset_on_change: true,
            tool_use_enabled: true,
            gradient_contrast: 5.,
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
