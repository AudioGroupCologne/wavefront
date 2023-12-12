use spectrum_analyzer::scaling::scale_to_zero_to_one;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use crate::components::microphone::Microphone;
use crate::math::constants::*;

pub fn calc_mic_spectrum(microphone: &mut Microphone, delta_t: f32) -> Vec<[f64; 2]> {
    let samples = if microphone.record.len() < FFT_WINDOW_SIZE {
        let mut s = microphone.record.clone();
        s.resize(FFT_WINDOW_SIZE, [0.0, 0.0]);
        s
    } else {
        microphone.record[microphone.record.len() - FFT_WINDOW_SIZE..].to_vec()
    };

    let hann_window = hann_window(&samples.iter().map(|x| x[1] as f32).collect::<Vec<_>>());

    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        (1. / delta_t) as u32,
        FrequencyLimit::All,
        Some(&scale_to_zero_to_one),
    )
    .unwrap();

    let mapped_spectrum = spectrum_hann_window
        .data()
        .iter()
        // .map(|x| [x.0.val().log10() as f64, x.1.val() as f64])
        .map(|(x, y)| [x.val() as f64, y.val() as f64])
        .collect::<Vec<_>>();

    microphone.spektrum.push(mapped_spectrum.clone());
    if microphone.spektrum.len() > 500 {
        //TODO: hardcode
        microphone.spektrum.remove(0);
    }

    mapped_spectrum
}
