// Following code is basically copied from the scipy signal package
// https://github.com/scipy/scipy/blob/main/scipy/signal/_filter_design.py

use bevy::prelude::*;

use super::constants::{BUTTERWORTH_N, PROPAGATION_SPEED};

#[derive(Debug, Resource)]
pub struct ButterFilter {
    sos: Vec<[f32; 5]>,
}

impl ButterFilter {
    pub fn calc(&mut self, delta_l: f32) {
        let sample_frequency = 1. / (delta_l / PROPAGATION_SPEED);
        let crit_freq = 20000f32.min(sample_frequency / 2.);
        let norm_crit_freq = crit_freq / (sample_frequency / 2.);

        // get analog lowpass prototype
        let (z, p, k) = buttap(BUTTERWORTH_N);
    }
    pub fn sos_filter(&self, data: Vec<f32>) {}
}

/// return zero, pole, gain for analog prototype of an Nth order Butterworth filter
fn buttap(n: u32) -> (f32, f32, f32) {
    (0., 0., 0.)
}
