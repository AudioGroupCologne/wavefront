use bevy::ecs::component::Component;
use bevy_pixel_buffer::bevy_egui::egui::emath::Pos2;
use bevy_pixel_buffer::bevy_egui::egui::{Color32, Rect};

#[derive(Component)]
pub struct Drag;

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Component, Debug)]
pub struct MenuSelected;
