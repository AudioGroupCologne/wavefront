use bevy::prelude::*;

#[derive(Debug, Component)]
/// A wall component containing the index of the corresponding cell in the grid
pub struct Wall(pub usize);
