use bevy::prelude::*;
use bevy_file_dialog::FileDialogExt;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;
use crate::ui::loading::SaveFileContents;
use crate::ui::state::{SimTime, UiState};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (update_wall_event, reset_event, save_event, load_event),
        )
        .add_event::<UpdateWalls>()
        .add_event::<Reset>()
        .add_event::<Load>()
        .add_event::<Save>();
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

#[derive(Event, Default)]
pub struct Reset {
    pub force: bool,
}

pub fn reset_event(
    mut reset_ev: EventReader<Reset>,
    mut grid: ResMut<Grid>,
    mut sim_time: ResMut<SimTime>,
    ui_state: Res<UiState>,
    mut mics: Query<&mut Microphone>,
) {
    for r in reset_ev.read() {
        if ui_state.reset_on_change || r.force {
            sim_time.time_since_start = 0f32;
            grid.reset_cells(ui_state.boundary_width);
            mics.iter_mut().for_each(|mut mic| mic.clear());
        }
    }
}

#[derive(Event)]
pub struct Save;

pub fn save_event(
    mut commands: Commands,
    mut save_ev: EventReader<Save>,
    sources: Query<&Source>,
    mics: Query<&Microphone>,
    rect_walls: Query<&RectWall>,
    circ_walls: Query<&CircWall>,
    gradient: Res<Gradient>,
) {
    for _ in save_ev.read() {
        let sources = sources.iter().collect::<Vec<_>>();
        let mics = mics.iter().collect::<Vec<_>>();
        let rect_walls = rect_walls.iter().collect::<Vec<_>>();
        let circ_walls = circ_walls.iter().collect::<Vec<_>>();

        let data =
            crate::ui::saving::serialize(&sources, &mics, &rect_walls, &circ_walls, &gradient)
                .unwrap();

        commands
            .dialog()
            .add_filter("JSON", &["json"])
            .set_file_name("save.json")
            .set_directory("./")
            .set_title("Select a file to save to")
            .save_file::<SaveFileContents>(data);
    }
}

#[derive(Event)]
pub struct Load;

pub fn load_event(mut commands: Commands, mut load_ev: EventReader<Load>) {
    for _ in load_ev.read() {
        commands
            .dialog()
            .add_filter("JSON", &["json"])
            .set_directory("./")
            .set_title("Select a file to load")
            .load_file::<SaveFileContents>();
    }
}
