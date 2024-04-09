use bevy::prelude::*;

pub fn update_fps(
    fixed_timestep: ResMut<Time<Fixed>>,
    mut last_time: Local<f64>,
    mut fps: ResMut<Fps>,
) {
    fps.0 = 1. / (fixed_timestep.elapsed_seconds_f64() - *last_time);
    *last_time = fixed_timestep.elapsed_seconds_f64();
}

#[derive(Debug, Resource, Default)]
pub struct Fps(pub f64);
