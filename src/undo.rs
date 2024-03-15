use bevy::prelude::*;
use egui::util::undoer::Undoer;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};
use crate::events::UpdateWalls;
use crate::grid::plugin::ComponentIDs;
use crate::ui::state::UiState;

/// The undo resource. This is a wrapper around the [`Undoer`] struct from the [`egui`] crate.
#[derive(Resource, Default)]
pub struct Undo(Undoer<State>);

/// The state of the application. This is used for undo/redo.
#[derive(Default, PartialEq, Clone)]
struct State {
    sources: Vec<Source>,
    mics: Vec<Microphone>,
    rect_walls: Vec<RectWall>,
    circle_walls: Vec<CircWall>,
    ui_state: UiState,
    ids: ComponentIDs,
}

/// Manages the undo/redo functionality.
pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Undo>()
            .add_systems(Update, undo_redo)
            .add_systems(PostUpdate, update_state);
    }
}

/// Feeds the current state into the undoer.
fn update_state(
    mut undo: ResMut<Undo>,
    ui_state: Res<UiState>,
    sources: Query<&Source>,
    mics: Query<&Microphone>,
    rect_walls: Query<&RectWall>,
    circle_walls: Query<&CircWall>,
    ids: Res<ComponentIDs>,
    time: Res<Time>,
) {
    let sources = sources.iter().copied().collect::<Vec<_>>();
    let mics = mics
        .iter()
        .map(|mic| Microphone::new(mic.x, mic.y, mic.id))
        .collect::<Vec<_>>();
    let rect_walls = rect_walls.iter().copied().collect::<Vec<_>>();
    let circle_walls = circle_walls.iter().copied().collect::<Vec<_>>();

    let state = State {
        sources,
        mics,
        rect_walls,
        circle_walls,
        ids: *ids,
        ui_state: *ui_state,
    };

    undo.0.feed_state(time.elapsed_seconds_f64(), &state);
}

/// Updates the state of the application based on the undo/redo commands.
fn undo_redo(
    mut undo: ResMut<Undo>,
    mut ui_state: ResMut<UiState>,
    mut ids: ResMut<ComponentIDs>,
    mut commands: Commands,
    mut wall_update_ev: EventWriter<UpdateWalls>,
    q_sources: Query<(Entity, &Source)>,
    q_mics: Query<(Entity, &Microphone)>,
    q_rect_walls: Query<(Entity, &RectWall)>,
    q_circle_walls: Query<(Entity, &CircWall)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    #[cfg(not(target_os = "macos"))]
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    #[cfg(target_os = "macos")]
    let ctrl = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    // on qwertz keyboards this binds to the z key
    let is_undo = ctrl && !shift && keys.just_pressed(KeyCode::KeyY);
    let is_redo = ctrl && shift && keys.just_pressed(KeyCode::KeyY);

    if is_undo || is_redo {
        let sources = q_sources.iter().map(|x| *x.1).collect::<Vec<_>>();
        let mics = q_mics.iter().map(|x| x.1.clone()).collect::<Vec<_>>();
        let rect_walls = q_rect_walls.iter().map(|x| *x.1).collect::<Vec<_>>();
        let circle_walls = q_circle_walls.iter().map(|x| *x.1).collect::<Vec<_>>();

        let current_state = State {
            sources,
            mics,
            rect_walls,
            circle_walls,
            ids: *ids,
            ui_state: *ui_state,
        };

        let new_state = if is_undo {
            undo.0.undo(&current_state)
        } else {
            undo.0.redo(&current_state)
        };

        if let Some(state) = new_state {
            for (e, _) in q_sources.iter() {
                commands.entity(e).despawn();
            }
            for (e, _) in q_mics.iter() {
                commands.entity(e).despawn();
            }
            for (e, _) in q_rect_walls.iter() {
                commands.entity(e).despawn();
            }
            for (e, _) in q_circle_walls.iter() {
                commands.entity(e).despawn();
            }

            for source in &state.sources {
                commands.spawn(*source);
            }
            for mic in &state.mics {
                commands.spawn(mic.clone());
            }
            for rect_wall in &state.rect_walls {
                commands.spawn(*rect_wall);
            }
            for circ_wall in &state.circle_walls {
                commands.spawn(*circ_wall);
            }

            wall_update_ev.send(UpdateWalls);

            *ids = state.ids;
            *ui_state = state.ui_state;
        }
    }
}
