use bevy::prelude::*;
use egui::util::undoer::Undoer;

use crate::components::microphone::Microphone;
use crate::components::source::Source;
use crate::components::wall::{CircWall, RectWall};
use crate::events::{Reset, UpdateWalls};
use crate::grid::plugin::ComponentIDs;

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
    ids: ComponentIDs,
}

/// Manages the undo/redo functionality.
pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Undo>()
            .add_systems(
                PostUpdate,
                (undo_redo_key, undo_event, update_state).chain(),
            )
            .add_event::<UndoEvent>();
    }
}

/// Feeds the current state into the undoer.
fn update_state(
    mut undo: ResMut<Undo>,
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
    };

    undo.0.feed_state(time.elapsed_seconds_f64(), &state);
}

/// Updates the state of the application based on the undo/redo commands.
fn undo_redo_key(keys: Res<ButtonInput<KeyCode>>, mut undo_ev: EventWriter<UndoEvent>) {
    #[cfg(not(target_os = "macos"))]
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    #[cfg(target_os = "macos")]
    let ctrl = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    // on qwertz keyboards this binds to the z key
    if ctrl && !shift && keys.just_pressed(KeyCode::KeyY) {
        undo_ev.send(UndoEvent(true));
    } else if ctrl && shift && keys.just_pressed(KeyCode::KeyY) {
        undo_ev.send(UndoEvent(false));
    }
}

// bool: true -> undo, false -> redo
// TODO: make better
#[derive(Event)]
pub struct UndoEvent(pub bool);

pub fn undo_event(
    mut undo_ev: EventReader<UndoEvent>,
    mut undo: ResMut<Undo>,
    mut ids: ResMut<ComponentIDs>,
    mut commands: Commands,
    mut wall_update_ev: EventWriter<UpdateWalls>,
    mut reset_ev: EventWriter<Reset>,
    q_sources: Query<(Entity, &Source)>,
    q_mics: Query<(Entity, &Microphone)>,
    q_rect_walls: Query<(Entity, &RectWall)>,
    q_circle_walls: Query<(Entity, &CircWall)>,
) {
    for event in undo_ev.read() {
        println!("{}", event.0);
        let sources = q_sources.iter().map(|x| *x.1).collect::<Vec<_>>();
        let mics = q_mics
            .iter()
            .map(|(_, mic)| Microphone::new(mic.x, mic.y, mic.id))
            .collect::<Vec<_>>();
        let rect_walls = q_rect_walls.iter().map(|x| *x.1).collect::<Vec<_>>();
        let circle_walls = q_circle_walls.iter().map(|x| *x.1).collect::<Vec<_>>();

        let current_state = State {
            sources,
            mics,
            rect_walls,
            circle_walls,
            ids: *ids,
        };

        // get new state based on undo/redo and the current state
        let new_state = if event.0 {
            undo.0.undo(&current_state)
        } else {
            undo.0.redo(&current_state)
        };

        // if there is a new state, update the entities
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
            reset_ev.send(Reset);

            *ids = state.ids;
        }
    }
}
