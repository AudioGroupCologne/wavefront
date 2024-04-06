use spectrum_analyzer::scaling::{scale_20_times_log10, scale_to_zero_to_one};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use crate::components::microphone::Microphone;
use crate::math::constants::*;
use crate::ui::state::FftScaling;

/// Calculate the spectrum of a [`Microphone`] based on the record field.
/// The spectrum is calculated using the FFT algorithm and a Hann window. The corresponding window size is specified in [`FFT_WINDOW_SIZE`].
/// The spectrum is then mapped to a logarithmic scale on the frequency axis and returned.
/// * `microphone` - The microphone to calculate the spectrum for.
/// * `ui_state` - The current state of the UI.
/// * `scaling` - The scaling to apply to the spectrum on the amplitude axis.
pub fn calc_mic_spectrum(
    microphone: &Microphone,
    scaling: FftScaling,
    delta_t: f32,
) -> Vec<[f64; 2]> {
    let samples = if microphone.record.len() < FFT_WINDOW_SIZE {
        let mut s = microphone.record.clone();
        s.resize(FFT_WINDOW_SIZE, [0.0, 0.0]);
        s
    } else {
        microphone.record[microphone.record.len() - FFT_WINDOW_SIZE..].to_vec()
    };

    let hann_window = hann_window(&samples.iter().map(|x| x[1] as f32).collect::<Vec<_>>());
    // always returns frequencies up to sampling_rate/2
    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        // "Normally" the sample rate should be calculated as: (1. / delta_t) as u32
        // But because the rate of samples provided does not actually change when changing delta_l,
        // we can just use the constant value. Calculated as 1/(0.001 / 343) = 343200
        (1. / delta_t) as u32,
        FrequencyLimit::All,
        match scaling {
            FftScaling::ZeroToOne => Some(&scale_to_zero_to_one),
            FftScaling::Decibels => Some(&scale_20_times_log10),
        },
    )
    .unwrap();

    let mapped_spectrum = spectrum_hann_window
        .data()
        .iter()
        .filter_map(|(x, y)| {
            // only return values between 0 and 20_000 Hz
            if x.val() > 0. && x.val() < 20_000. {
                Some([x.val().log10() as f64, y.val() as f64])
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    mapped_spectrum
}
