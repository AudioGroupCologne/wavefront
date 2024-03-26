use bevy::ecs::system::{Commands, Resource};
use bevy::math::UVec2;
use bevy_file_dialog::FileDialogExt;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;
use bevy_pixel_buffer::query::PixelBuffersItem;
use egui_plot::{GridMark, Line, Plot, PlotPoints};
use plotters::prelude::*;

use super::dialog::SaveFileContents;
use super::state::FftMicrophone;
use crate::components::microphone::Microphone;
use crate::math::fft::calc_mic_spectrum;

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
    pub mics: &'a [&'a Microphone],
    pub pixel_buffer: &'a mut PixelBuffersItem<'a>,
    pub fft_microphone: &'a mut FftMicrophone,
    pub commands: &'a mut Commands<'a, 'a>,
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
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    if ui
                        .button("Screenshot Plot")
                        .on_hover_text("Save a screenshot of the plot")
                        .clicked()
                    {
                        let colors = [
                            RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, BLACK, WHITE
                        ];

                        // let mut v = ContinuousView::new();
                        // let mut mics = self.mics.to_vec();
                        // mics.sort_by_cached_key(|mic| mic.id);
                        // for (index, mic) in mics.iter().enumerate() {
                        //     //TODO: because of this clone, the app is getting slower as time goes on (because the vec is getting bigger)
                        //     let l1 = plotlib::repr::Plot::new(
                        //         mic.record.iter().map(|x| (x[0], x[1])).collect(),
                        //     )
                        //     .line_style(
                        //         LineStyle::new()
                        //             .colour(colors[index % (colors.len() - 1)])
                        //             .linejoin(LineJoin::Round)
                        //             .width(1.),
                        //     )
                        //     .legend(format!(
                        //         "Microphone {} (x: {}, y: {})",
                        //         mic.id, mic.x, mic.y
                        //     ));

                        //     v = v.add(l1);
                        // }

                        // v = v.y_label("Amplitude").x_label("Time (s)");

                        // let data = Page::single(&v)
                        //     .to_svg()
                        //     .expect("correct svg")
                        //     .to_string()
                        //     .into_bytes();

                        let mut string_buffer = String::new();
                        {
                            let mut mics = self.mics.to_vec();
                            mics.sort_by_cached_key(|mic| mic.id);
                            let highest_x = mics.iter().map(|mic| {
                                *mic.record
                                    .iter()
                                    .map(|x| x[0])
                                    .collect::<Vec<_>>()
                                    .last()
                                    .unwrap_or(&0.)
                            }).reduce(f64::max).unwrap_or(0.) as f32;

                            let highest_y = mics.iter().map(|mic| {
                                *mic.record
                                    .iter()
                                    .map(|x| x[1])
                                    .collect::<Vec<_>>()
                                    .last()
                                    .unwrap_or(&0.)
                            }).reduce(f64::max).unwrap_or(0.).abs() as f32;

                            let root = SVGBackend::with_string(&mut string_buffer, (640, 480))
                                .into_drawing_area();
                            root.fill(&WHITE).unwrap();
                            let root = root.margin(10, 10, 10, 10);
                            // After this point, we should be able to construct a chart context
                            let mut chart = ChartBuilder::on(&root)
                                // Set the size of the label region
                                .x_label_area_size(20)
                                .y_label_area_size(40)
                                // Finally attach a coordinate on the drawing area and make a chart context
                                .build_cartesian_2d(0f32..highest_x, -highest_y..highest_y)
                                .unwrap();

                            // Then we can draw a mesh
                            chart
                                .configure_mesh()
                                // We can customize the maximum number of labels allowed for each axis
                                .x_labels(5)
                                .y_labels(5)
                                // We can also change the format of the label text
                                .y_label_formatter(&|x| format!("{:.3}", x))
                                .draw()
                                .unwrap();

                            for (index, &mic) in mics.iter().enumerate() {
                                let points = mic
                                    .record
                                    .iter()
                                    .map(|x| (x[0] as f32, x[1] as f32))
                                    .collect::<Vec<_>>();
                                // draw something in the drawing area
                                chart
                                    .draw_series(LineSeries::new(points, &colors[index % (colors.len() - 1)]))
                                    .unwrap()
                                    .label(format!(
                                        "Microphone {} (x: {}, y: {})",
                                        mic.id, mic.x, mic.y
                                    ))
                                    .legend(move |(x, y)| {
                                        // I don't get this
                                        PathElement::new(vec![(x, y), (x + 20, y)], &colors[index % (colors.len() - 1)])
                                    });
                            }

                            chart
                                .configure_series_labels()
                                .background_style(&WHITE.mix(0.8))
                                .border_style(&BLACK)
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

                ui.separator();

                Plot::new("mic_plot")
                    .allow_zoom([true, true])
                    .x_axis_label("Time (s)")
                    .y_axis_label("Amplitude")
                    .label_formatter(|_, value| {
                        format!("Amplitude: {:.2}\nTime: {:.4} s", value.y, value.x)
                    })
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        // TODO: allocation here is not very nice
                        let mut mics = self.mics.to_vec();
                        mics.sort_by_cached_key(|mic| mic.id);
                        for mic in mics {
                            //TODO: because of this clone, the app is getting slower as time goes on (because the vec is getting bigger)
                            let points = PlotPoints::new(mic.record.clone());
                            let line = Line::new(points);
                            plot_ui.line(line.name(format!(
                                "Microphone {} (x: {}, y: {})",
                                mic.id, mic.x, mic.y
                            )));
                        }
                    });
            }
            Tab::Frequency => {
                egui::ComboBox::from_label("FFT Microphone")
                    .selected_text(if let Some(index) = self.fft_microphone.mic_id {
                        format!("Microphone {index}")
                    } else {
                        "No Microphone Selected".to_string()
                    })
                    .show_ui(ui, |ui| {
                        for mic in self.mics {
                            ui.selectable_value(
                                &mut self.fft_microphone.mic_id,
                                Some(mic.id),
                                format!("Microphone {}", mic.id),
                            );
                        }
                    });

                ui.separator();
                Plot::new("fft_plot")
                    .allow_zoom([false, false])
                    .allow_scroll(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .x_axis_label("Frequency (Hz)")
                    .y_axis_label("Intensity (dB)")
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
                    .label_formatter(|_, value| {
                        format!(
                            "Intensity: {:.2} dB\nFrequency: {:.2} Hz",
                            value.y,
                            10_f64.powf(value.x)
                        )
                    })
                    .show(ui, |plot_ui| {
                        if self.fft_microphone.mic_id.is_none() {
                            return;
                        }

                        if let Some(mic) = self
                            .mics
                            .iter()
                            .find(|m| m.id == self.fft_microphone.mic_id.expect("no mic selected"))
                        {
                            let mapped_spectrum = calc_mic_spectrum(mic);
                            // remove the first element, because of log it is at x=-inf
                            let mapped_spectrum = &mapped_spectrum[1..];

                            let points = PlotPoints::new(mapped_spectrum.to_vec());
                            let line = Line::new(points);
                            plot_ui.line(line);
                        }
                    });
            }
            Tab::Spectrogram => {
                egui::ComboBox::from_label("FFT Microphone")
                    .selected_text(if let Some(index) = self.fft_microphone.mic_id {
                        format!("Microphone {index}")
                    } else {
                        "No Microphone Selected".to_string()
                    })
                    .show_ui(ui, |ui| {
                        for mic in self.mics {
                            ui.selectable_value(
                                &mut self.fft_microphone.mic_id,
                                Some(mic.id),
                                format!("Microphone {}", mic.id),
                            );
                        }
                    });
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
