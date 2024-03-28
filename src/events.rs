use bevy::prelude::*;
use bevy::window::PresentMode;

use crate::components::wall::{CircWall, RectWall};
use crate::grid::grid::Grid;
use crate::ui::state::{SimTime, UiState};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (update_wall_event, reset_event, vsync_event))
            .add_event::<UpdateWalls>()
            .add_event::<Vsync>()
            .add_event::<Reset>();
    }
}

#[derive(Event)]
pub struct UpdateWalls;

pub fn update_wall_event(
    mut wall_update_ev: EventReader<UpdateWalls>,
    mut grid: ResMut<Grid>,
    ui_state: Res<UiState>,
    rect_walls: Query<&RectWall>,
    circ_walls: Query<&CircWall>,
) {
    for _ in wall_update_ev.read() {
        grid.update_walls(&rect_walls, &circ_walls, ui_state.boundary_width);
    }
}

#[derive(Event)]
pub struct Reset;

pub fn reset_event(
    mut reset_ev: EventReader<Reset>,
    mut grid: ResMut<Grid>,
    mut sim_time: ResMut<SimTime>,
    ui_state: Res<UiState>,
) {
    if ui_state.reset_on_change {
        for _ in reset_ev.read() {
            sim_time.time_since_start = 0f32;
            grid.reset_cells(ui_state.boundary_width);
        }
    }
}

#[derive(Event)]
pub struct Vsync;

pub fn vsync_event(mut vsync_ev: EventReader<Vsync>, mut windows: Query<&mut Window>) {
    for _ in vsync_ev.read() {
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
    }
}
