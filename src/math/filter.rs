// Following code is basically copied from the scipy signal package
// https://github.com/scipy/scipy/blob/main/scipy/signal/_filter_design.py

use bevy::prelude::*;
use butterworth::{Cutoff, Filter};
use rand::{thread_rng, Rng};

use super::constants::{BUTTERWORTH_N, DEFAULT_DELTA_L, PROPAGATION_SPEED};

#[derive(Debug, Resource)]
pub struct ButterFilter {
    pub filter: Filter,
}

impl Default for ButterFilter {
    fn default() -> Self {
        let sample_frequency = 1. / (DEFAULT_DELTA_L / PROPAGATION_SPEED);
        let crit_freq = 20000f32.min(sample_frequency / 2.);
        let crit_freq = 15000f32;

        let filter = Filter::new(
            BUTTERWORTH_N,
            sample_frequency as f64,
            Cutoff::LowPass(crit_freq as f64),
        )
        .unwrap();

        let mut wn_vec: Vec<f64> = vec![];
        for _ in 0..1000 {
            wn_vec.push(thread_rng().sample::<f64, _>(rand_distr::StandardNormal));
        }

        let res = filter.bidirectional(&wn_vec).unwrap();
        println!("{:?}", res);

        Self { filter }
    }
}

impl ButterFilter {
    pub fn re_calc(&mut self, delta_l: f32) {
        let sample_frequency = 1. / (delta_l / PROPAGATION_SPEED) as f64;
        let crit_freq = 20000f64.min(sample_frequency / 2.000000001);

        self.filter =
            Filter::new(BUTTERWORTH_N, sample_frequency, Cutoff::LowPass(crit_freq)).unwrap();
    }
}
