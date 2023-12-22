use bevy::prelude::*;

#[derive(Debug, Default, Component)]
/// A microphone on the grid that records the pressure at its position
pub struct Microphone {
    pub x: u32,
    pub y: u32,
    ///TODO: think of a better id system, right now we are just counting up
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

    pub fn spawn_initial_microphones(mut commands: Commands) {
        commands.spawn(Microphone::new(250, 250, 1));
        commands.spawn(Microphone::new(100, 100, 2));
        commands.spawn(Microphone::new(650, 650, 3));
    }

    pub fn clear(&mut self) {
        self.record = vec![[0., 0.]];
    }
}
