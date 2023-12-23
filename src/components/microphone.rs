use bevy::prelude::*;

use crate::grid::plugin::ComponentIDs;

#[derive(Debug, Default, Component)]
/// A microphone on the grid that records the pressure at its position
pub struct Microphone {
    pub x: u32,
    pub y: u32,
    pub id: usize,
    pub record: Vec<[f64; 2]>,
    pub spectrum: Vec<Vec<[f64; 2]>>,
}

impl Microphone {
    pub fn new(x: u32, y: u32, id: usize) -> Self {
        Self {
            x,
            y,
            id,
            record: vec![[0., 0.]],
            spectrum: vec![],
        }
    }

    pub fn spawn_initial_microphones(
        mut commands: Commands,
        mut component_ids: ResMut<ComponentIDs>,
    ) {
        commands.spawn(Microphone::new(
            250,
            250,
            component_ids.get_current_mic_id(),
        ));
        commands.spawn(Microphone::new(
            100,
            100,
            component_ids.get_current_mic_id(),
        ));
        commands.spawn(Microphone::new(
            650,
            650,
            component_ids.get_current_mic_id(),
        ));
    }

    pub fn clear(&mut self) {
        self.record = vec![[0., 0.]];
    }
}
