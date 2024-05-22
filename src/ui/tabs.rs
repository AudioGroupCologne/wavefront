use bevy::ecs::system::{Commands, Resource};
use bevy::math::UVec2;
use bevy_file_dialog::FileDialogExt;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;
use bevy_pixel_buffer::query::PixelBuffersItem;
use egui_plot::{GridMark, Line, Plot, PlotBounds, PlotPoints};
use plotters::prelude::*;

use super::loading::SaveFileContents;
use super::state::{FftScaling, UiState};
use crate::components::microphone::Microphone;
use crate::math::fft::calc_mic_spectrum;
use crate::math::transformations::interpolate;

#[derive(Resource)]
pub struct DockState {
    pub tree: egui_dock::DockState<Tab>,
}

impl Default for DockState {
    fn default() -> Self {
        let tree = create_tree();
        Self { tree }
    }
}

pub enum Tab {
    Volume,
    Frequency,
    Spectrogram,
}

pub struct PlotTabs<'a> {
    mics: &'a mut Vec<&'a mut Microphone>,
    pixel_buffer: &'a mut PixelBuffersItem<'a>,
    commands: &'a mut Commands<'a, 'a>,
    delta_t: f32,
    sim_time: f64,
    delta_time: f64,
    ui_state: &'a mut UiState,
}

impl<'a> PlotTabs<'a> {
    pub fn new(
        mics: &'a mut Vec<&'a mut Microphone>,
        pixel_buffer: &'a mut PixelBuffersItem<'a>,
        commands: &'a mut Commands<'a, 'a>,
        delta_t: f32,
        sim_time: f64,
        delta_time: f64,
        ui_state: &'a mut UiState,
    ) -> Self {
        Self {
            mics,
            pixel_buffer,
            commands,
            delta_t,
            sim_time,
            ui_state,
            delta_time,
        }
    }
}

impl<'a> egui_dock::TabViewer for PlotTabs<'a> {
    type Tab = Tab;
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Volume => "Volume".into(),
            Tab::Frequency => "Frequency".into(),
            Tab::Spectrogram => "Spectrogram".into(),
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Volume => {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.ui_state.scroll_volume_plot, "Scroll Volume Plot");

                    ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        if ui
                            .button("Export to SVG")
                            .on_hover_text("Save the plot to an SVG file")
                            .clicked()
                        {
                            let colors = [RED, BLUE, GREEN, CYAN, MAGENTA, BLACK, WHITE];

                            let mut string_buffer = String::new();
                            {
                                // find the highest x and y values to set the plot size
                                let highest_x =
                                    self.mics
                                        .iter()
                                        .map(|mic| {
                                            *mic.record
                                                .iter()
                                                .map(|x| x[0])
                                                .collect::<Vec<_>>()
                                                .last()
                                                .unwrap_or(&0.)
                                        })
                                        .reduce(f64::max)
                                        .unwrap_or(0.) as f32;

                                let highest_y = self
                                    .mics
                                    .iter()
                                    .map(|mic| {
                                        mic.record
                                            .iter()
                                            .map(|x| x[1])
                                            .reduce(f64::max)
                                            .unwrap_or(0.)
                                    })
                                    .reduce(f64::max)
                                    .unwrap_or(0.)
                                    .abs() as f32;

                                let longest_recording = self
                                    .mics
                                    .iter()
                                    .map(|mic| mic.record.len())
                                    .reduce(usize::max)
                                    .unwrap_or(0);

                                // TODO: the svg now gets longer and longer... maybe restrict to the scroll area when scrolling is enabled?
                                let root = SVGBackend::with_string(
                                    &mut string_buffer,
                                    (longest_recording as u32, 600),
                                )
                                .into_drawing_area();
                                root.fill(&WHITE).unwrap();
                                let root = root.margin(10, 10, 10, 10);

                                let mut chart = ChartBuilder::on(&root)
                                    .x_label_area_size(40)
                                    .y_label_area_size(50)
                                    .build_cartesian_2d(0f32..highest_x, -highest_y..highest_y)
                                    .unwrap();

                                chart
                                    .configure_mesh()
                                    .x_labels(5)
                                    .y_labels(5)
                                    .y_label_formatter(&|x| format!("{:.2}", x))
                                    .y_desc("Amplitude")
                                    .x_desc("Simulation Time (s)")
                                    .draw()
                                    .unwrap();

                                for (index, ref mic) in self.mics.iter().enumerate() {
                                    let points = mic
                                        .record
                                        .iter()
                                        .map(|x| (x[0] as f32, x[1] as f32))
                                        .collect::<Vec<_>>();

                                    chart
                                        .draw_series(LineSeries::new(
                                            points,
                                            colors[index % (colors.len() - 1)],
                                        ))
                                        .unwrap()
                                        .label(format!(
                                            "Microphone {} (x: {}, y: {})",
                                            mic.id, mic.x, mic.y
                                        ))
                                        .legend(move |(x, y)| {
                                            PathElement::new(
                                                vec![(x, y), (x + 20, y)],
                                                colors[index % (colors.len() - 1)],
                                            )
                                        });
                                }

                                chart
                                    .configure_series_labels()
                                    .position(SeriesLabelPosition::UpperRight)
                                    .background_style(WHITE.mix(0.8))
                                    .border_style(BLACK)
                                    .draw()
                                    .unwrap();
                            }

                            self.commands
                                .dialog()
                                .add_filter("SVG", &["svg"])
                                .set_file_name("function.svg")
                                .set_directory("./")
                                .set_title("Select a file to save to")
                                .save_file::<SaveFileContents>(string_buffer.into_bytes());
                        }
                    });
                });

                ui.separator();

                let scroll_volume_plot = self.ui_state.scroll_volume_plot;

                Plot::new("mic_plot")
                    .allow_zoom([!scroll_volume_plot, !scroll_volume_plot])
                    .allow_drag(!scroll_volume_plot)
                    .allow_scroll(!scroll_volume_plot)
                    .x_axis_label("Simulation Time (ms)")
                    .y_axis_label("Amplitude")
                    .label_formatter(|_, value| {
                        format!("Amplitude: {:.2}\nTime: {:.4} ms", value.y, value.x)
                    })
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        if scroll_volume_plot {
                            let highest_x = self.sim_time;
                            let highest_y = self
                                .mics
                                .iter()
                                .map(|mic| mic.record.last().unwrap_or(&[0., 0.])[1])
                                .reduce(f64::max)
                                .unwrap_or(0.)
                                .abs();

                            if highest_y > self.ui_state.highest_y_volume_plot {
                                self.ui_state.highest_y_volume_plot = highest_y;
                            }

                            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                [
                                    highest_x * 1000. - 5.,
                                    -(self.ui_state.highest_y_volume_plot + 0.2),
                                ],
                                [highest_x * 1000., self.ui_state.highest_y_volume_plot + 0.2],
                            ));
                        }

                        for mic in &mut *self.mics {
                            let values = mic.record.iter().map(|x| [x[0] * 1000., x[1]]).collect();
                            let points = PlotPoints::new(values);
                            let line = Line::new(points);
                            plot_ui.line(line.name(format!(
                                "Microphone {} (x: {}, y: {})",
                                mic.id, mic.x, mic.y
                            )));
                        }
                    });
            }
            Tab::Frequency => {
                ui.horizontal(|ui| {
                    ui.menu_button("FFT Microhones", |ui| {
                        for mic in &mut *self.mics {
                            ui.checkbox(&mut mic.show_fft, format!("Microphone {}", mic.id));
                        }
                    });

                    ui.add(egui::Separator::default().vertical());

                    egui::ComboBox::from_label("Scaling")
                        .selected_text(self.ui_state.fft_scaling.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.ui_state.fft_scaling,
                                FftScaling::Normalized,
                                format!("{}", FftScaling::Normalized),
                            );
                            ui.selectable_value(
                                &mut self.ui_state.fft_scaling,
                                FftScaling::Decibels,
                                format!("{}", FftScaling::Decibels),
                            );
                        });

                    ui.add(egui::Separator::default().vertical());

                    ui.checkbox(&mut self.ui_state.show_fft_approx, "Show Approximation");

                    ui.add(egui::Separator::default().vertical());

                    egui::ComboBox::from_label("FFT Window Size")
                        .selected_text(self.ui_state.fft_window_size.to_string())
                        .show_ui(ui, |ui| {
                            for window_size in [256, 512, 1024, 2048, 4096, 8192] {
                                ui.selectable_value(
                                    &mut self.ui_state.fft_window_size,
                                    window_size,
                                    format!("{}", window_size),
                                );
                            }
                        });
                });

                ui.separator();

                let unit = match self.ui_state.fft_scaling {
                    FftScaling::Normalized => "",
                    FftScaling::Decibels => "(dB)",
                };
                Plot::new("fft_plot")
                    .allow_zoom([false, false])
                    .allow_scroll(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .x_axis_label("Frequency (Hz)")
                    .y_axis_label(format!("Intensity {}", unit))
                    .legend(egui_plot::Legend::default())
                    .x_grid_spacer(|input| {
                        let mut marks = Vec::with_capacity(
                            input.bounds.1 as usize - input.bounds.0 as usize + 1,
                        );

                        for i in input.bounds.0 as u32 + 1..=input.bounds.1 as u32 {
                            marks.push(GridMark {
                                value: i as f64,
                                step_size: 1.,
                            });
                        }
                        marks
                    })
                    .x_axis_formatter(|mark, _, _| format!("{:.0}", 10_f64.powf(mark.value)))
                    .label_formatter(move |_, value| {
                        format!(
                            "Intensity: {:.2} {}\nFrequency: {:.2} (Hz)",
                            value.y,
                            unit,
                            10_f64.powf(value.x)
                        )
                    })
                    .show(ui, |plot_ui| {
                        let mut current_highest_y = 0.;
                        let mut current_lowest_y = f64::MAX;
                        let mut current_lowest_x = f64::MAX;
                        let mut current_highest_x = 0.;

                        for mic in &mut *self.mics {
                            if !mic.show_fft {
                                continue;
                            }

                            let mapped_spectrum = calc_mic_spectrum(
                                mic,
                                self.ui_state.fft_scaling,
                                self.delta_t,
                                self.ui_state.fft_window_size,
                            );

                            // remove the first element, because of log it is at x=-inf
                            let mapped_spectrum = &mapped_spectrum[1..];

                            if self.ui_state.show_fft_approx {
                                let mut result = Vec::with_capacity(mapped_spectrum.len());

                                let n =
                                    (self.ui_state.fft_window_size as f64 / 256.).round() as i32;
                                for i in 0..mapped_spectrum.len() {
                                    let lower = if i as i32 - n < 0 {
                                        0usize
                                    } else {
                                        (i as i32 - n) as usize
                                    };
                                    let upper = if i as i32 + n >= mapped_spectrum.len() as i32 {
                                        mapped_spectrum.len() - 1
                                    } else {
                                        (i as i32 + n) as usize
                                    };

                                    let window = &mapped_spectrum[lower..upper];

                                    let m =
                                        window.iter().map(|x| x[1]).sum::<f64>() / (2 * n) as f64;
                                    result.push(m);
                                }

                                let points = PlotPoints::new(
                                    result
                                        .iter()
                                        .enumerate()
                                        .map(|(i, x)| [mapped_spectrum[i][0], *x])
                                        .collect(),
                                );
                                let line = Line::new(points);
                                plot_ui.line(line.name(format!("Approximation {}", mic.id)));
                            } else {
                                let points = PlotPoints::new(mapped_spectrum.to_vec());
                                let line = Line::new(points);
                                plot_ui.line(line.name(format!("Microphone {}", mic.id)));
                            }

                            let y_padding = match self.ui_state.fft_scaling {
                                FftScaling::Normalized => 0.05,
                                FftScaling::Decibels => 5.,
                            };

                            // set bounds based on the highest and lowest values (and interpolate to make it smooth)
                            let highest_y = mapped_spectrum
                                .iter()
                                .map(|x| x[1])
                                .reduce(f64::max)
                                .unwrap_or(0.)
                                + y_padding;
                            let highest_x = mapped_spectrum.last().unwrap_or(&[0., 0.])[0] + 0.1;
                            let lowest_x = mapped_spectrum.first().unwrap_or(&[0., 0.])[0] - 0.1;
                            let lowest_y = mapped_spectrum
                                .iter()
                                .map(|x| x[1])
                                .reduce(f64::min)
                                .unwrap_or(0.)
                                - y_padding;

                            if highest_y > current_highest_y {
                                current_highest_y = highest_y;
                            }

                            if lowest_y < current_lowest_y {
                                current_lowest_y = lowest_y;
                            }

                            if lowest_x < current_lowest_x {
                                current_lowest_x = lowest_x;
                            }

                            if highest_x > current_highest_x {
                                current_highest_x = highest_x;
                            }
                        }

                        const ANIMATION_DURATION: f64 = 0.2; // Duration in seconds
                        const UPDATE_RATE: f64 = 1.0 / ANIMATION_DURATION; // How fast to update based on duration

                        let interpolation_step = self.delta_time * UPDATE_RATE;
                        let current_bounds = plot_ui.plot_bounds();

                        let lowest_x = interpolate(
                            current_bounds.min()[0],
                            current_lowest_x,
                            interpolation_step,
                        );
                        let lowest_y = interpolate(
                            current_bounds.min()[1],
                            current_lowest_y,
                            interpolation_step,
                        );
                        let highest_x = interpolate(
                            current_bounds.max()[0],
                            current_highest_x,
                            interpolation_step,
                        );
                        let highest_y = interpolate(
                            current_bounds.max()[1],
                            current_highest_y,
                            interpolation_step,
                        );

                        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                            [lowest_x, lowest_y],
                            [highest_x, highest_y],
                        ));
                    });
            }
            Tab::Spectrogram => {
                if !self.ui_state.enable_spectrogram {
                    ui.add_space(20.);
                    ui.vertical_centered(|ui| ui.label("Spectrogram is currently experimental. You can enable it in the settings."));
                    return;
                }

                // TODO: fix spectrogram for multiple microphones
                ui.separator();

                let spectrum_size = ui.available_size();
                let texture = self.pixel_buffer.egui_texture();
                ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture.id, texture.size))
                        .shrink_to_fit(),
                );

                self.pixel_buffer.pixel_buffer.size = PixelBufferSize {
                    size: UVec2::new(spectrum_size.x as u32, spectrum_size.y as u32),
                    pixel_size: UVec2::new(1, 1),
                };
            }
        };
    }
}

pub fn create_tree() -> egui_dock::DockState<Tab> {
    egui_dock::DockState::new(vec![Tab::Volume, Tab::Frequency, Tab::Spectrogram])
}
