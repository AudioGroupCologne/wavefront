use bevy::ecs::component::Component;
use bevy_pixel_buffer::bevy_egui::egui::emath::Pos2;
use bevy_pixel_buffer::bevy_egui::egui::{Color32, Rect};

#[derive(Component)]
pub struct Overlay;

#[derive(Component)]
pub struct Drag;

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Component, Debug)]
pub struct MenuSelected;

pub trait Gizmo {
    fn get_gizmo_position(&self, rect: &Rect) -> Pos2;
    fn get_gizmo_color(&self) -> Color32;
}
