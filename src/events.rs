use bevy::prelude::*;
use bevy_file_dialog::FileDialogExt;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};
use crate::render::gradient::Gradient;
use crate::simulation::grid::Grid;
use crate::simulation::plugin::ComponentIDs;
use crate::ui::loading::SaveFileContents;
use crate::ui::state::{SimTime, UiState};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                update_wall_event,
                reset_event,
                save_event,
                load_event,
                new_event,
            ),
        )
        .add_event::<UpdateWalls>()
        .add_event::<Reset>()
        .add_event::<Load>()
        .add_event::<Save>()
        .add_event::<New>();
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

/// Event that resets the simulation. If force is set to true, it will override the `reset_on_change` toggle.
#[derive(Event, Default)]
pub struct Reset {
    pub force: bool,
}

pub fn reset_event(
    mut reset_ev: EventReader<Reset>,
    mut grid: ResMut<Grid>,
    mut sim_time: ResMut<SimTime>,
    mut ui_state: ResMut<UiState>,
    mut mics: Query<&mut Microphone>,
) {
    for r in reset_ev.read() {
        if ui_state.reset_on_change || r.force {
            sim_time.time_since_start = 0f32;
            sim_time.samples_since_start = 0;
            grid.reset_cells(ui_state.boundary_width);
            mics.iter_mut().for_each(|mut mic| mic.clear());
            ui_state.highest_y_volume_plot = 0f64;
        }
    }
}

#[derive(Event)]
pub struct New;

pub fn new_event(
    mut commands: Commands,
    mut new_ev: EventReader<New>,
    sources: Query<(Entity, &Source)>,
    mics: Query<(Entity, &Microphone)>,
    rect_walls: Query<(Entity, &RectWall)>,
    circ_walls: Query<(Entity, &CircWall)>,
    mut ui_state: ResMut<UiState>,
    mut grid: ResMut<Grid>,
    mut wall_update_ev: EventWriter<UpdateWalls>,
    mut fixed_timestep: ResMut<Time<Fixed>>,
    mut ids: ResMut<ComponentIDs>,
    mut gradient: ResMut<Gradient>,
    mut sim_time: ResMut<SimTime>,
) {
    for _ in new_ev.read() {
        for (e, _) in sources.iter() {
            commands.entity(e).despawn();
        }
        for (e, _) in rect_walls.iter() {
            commands.entity(e).despawn();
        }
        for (e, _) in circ_walls.iter() {
            commands.entity(e).despawn();
        }
        for (e, _) in mics.iter() {
            commands.entity(e).despawn();
        }

        grid.reset_cells(ui_state.boundary_width);
        wall_update_ev.send(UpdateWalls);
        *ui_state = UiState::default();
        fixed_timestep.set_timestep_hz(ui_state.framerate);
        ids.reset();
        *gradient = Gradient::default();
        sim_time.time_since_start = 0f32;
        sim_time.samples_since_start = 0;
        // TODO: clear undoer
    }
}

#[derive(Event)]
pub struct Save {
    pub new_file: bool,
}

pub fn save_event(
    mut commands: Commands,
    mut save_ev: EventReader<Save>,
    mut new_ev: EventWriter<New>,
    sources: Query<&Source>,
    mics: Query<&Microphone>,
    rect_walls: Query<&RectWall>,
    circ_walls: Query<&CircWall>,
    gradient: Res<Gradient>,
    ui_state: Res<UiState>,
) {
    for event in save_ev.read() {
        let sources = sources.iter().collect::<Vec<_>>();
        let mics = mics.iter().collect::<Vec<_>>();
        let rect_walls = rect_walls.iter().collect::<Vec<_>>();
        let circ_walls = circ_walls.iter().collect::<Vec<_>>();

        let data = crate::ui::saving::serialize(
            &sources,
            &mics,
            &rect_walls,
            &circ_walls,
            &gradient,
            ui_state.max_gradient,
            ui_state.min_gradient,
            ui_state.reset_on_change,
            ui_state.delta_l,
        )
        .unwrap();

        commands
            .dialog()
            .add_filter("JSON", &["json"])
            .set_file_name("save.json")
            .set_directory("./")
            .set_title("Select a file to save to")
            .save_file::<SaveFileContents>(data);

        if event.new_file {
            new_ev.send(New);
        }
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
