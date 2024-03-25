use bevy::ecs::system::{Commands, Resource};
use bevy::math::UVec2;
use bevy_file_dialog::FileDialogExt;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;
use bevy_pixel_buffer::query::PixelBuffersItem;
use egui_plot::{GridMark, Line, Plot, PlotPoints};
use plotlib::page::Page;
use plotlib::style::{LineJoin, LineStyle};
use plotlib::view::ContinuousView;

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
    Spectogram,
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
            Tab::Spectogram => "Spectogram".into(),
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
                            "aqua",
                            "aquamarine",
                            "blue",
                            "blueviolet",
                            "brown",
                            "burlywood",
                            "cadetblue",
                            "chartreuse",
                            "chocolate",
                            "coral",
                            "cornflowerblue",
                            "crimson",
                            "cyan",
                            "darkblue",
                            "darkcyan",
                            "darkgoldenrod",
                            "darkgray",
                            "darkgreen",
                            "darkkhaki",
                            "darkmagenta",
                            "darkolivegreen",
                            "darkorange",
                            "darkorchid",
                            "darkred",
                            "darksalmon",
                            "darkseagreen",
                            "darkslateblue",
                            "darkslategray",
                            "darkslategrey",
                            "darkturquoise",
                            "darkviolet",
                            "deeppink",
                            "deepskyblue",
                            "dimgray",
                            "dimgrey",
                            "dodgerblue",
                            "firebrick",
                            "forestgreen",
                            "fuchsia",
                            "gold",
                            "goldenrod",
                            "gray",
                            "green",
                            "greenyellow",
                            "hotpink",
                            "indianred",
                            "indigo",
                            "khaki",
                            "lawngreen",
                            "lightblue",
                            "lightcoral",
                            "lightgreen",
                            "lightpink",
                            "lightsalmon",
                            "lightseagreen",
                            "lightskyblue",
                            "lightsteelblue",
                            "lime",
                            "limegreen",
                            "magenta",
                            "maroon",
                            "mediumaquamarine",
                            "mediumblue",
                            "mediumorchid",
                            "mediumpurple",
                            "mediumseagreen",
                            "mediumslateblue",
                            "mediumspringgreen",
                            "mediumturquoise",
                            "mediumvioletred",
                            "midnightblue",
                            "navy",
                            "olive",
                            "olivedrab",
                            "orange",
                            "orangered",
                            "orchid",
                            "palegreen",
                            "paleturquoise",
                            "palevioletred",
                            "peru",
                            "pink",
                            "plum",
                            "powderblue",
                            "purple",
                            "rebeccapurple",
                            "red",
                            "rosybrown",
                            "royalblue",
                            "saddlebrown",
                            "salmon",
                            "sandybrown",
                            "seagreen",
                            "sienna",
                            "skyblue",
                            "slateblue",
                            "slategray",
                            "springgreen",
                            "steelblue",
                            "tan",
                            "teal",
                            "thistle",
                            "tomato",
                            "turquoise",
                            "violet",
                            "yellow",
                            "yellowgreen",
                        ];

                        let mut v = ContinuousView::new();
                        let mut mics = self.mics.to_vec();
                        mics.sort_by_cached_key(|mic| mic.id);
                        for (index, mic) in mics.iter().enumerate() {
                            //TODO: because of this clone, the app is getting slower as time goes on (because the vec is getting bigger)
                            let l1 = plotlib::repr::Plot::new(
                                mic.record.iter().map(|x| (x[0], x[1])).collect(),
                            )
                            .line_style(
                                LineStyle::new()
                                    .colour(colors[index % (colors.len() - 1)])
                                    .linejoin(LineJoin::Round)
                                    .width(1.),
                            )
                            .legend(format!(
                                "Microphone {} (x: {}, y: {})",
                                mic.id, mic.x, mic.y
                            ));

                            v = v.add(l1);
                        }

                        v = v.y_label("Amplitude").x_label("Time (s)");

                        let data = Page::single(&v)
                            .to_svg()
                            .expect("correct svg")
                            .to_string()
                            .into_bytes();

                        self.commands
                            .dialog()
                            .add_filter("SVG", &["svg"])
                            .set_file_name("function.svg")
                            .set_directory("./")
                            .set_title("Select a file to save to")
                            .save_file::<SaveFileContents>(data);
                    }
                });

                ui.add_space(5.);

                Plot::new("mic_plot")
                    .allow_zoom([true, false])
                    // .allow_scroll(false)
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
            Tab::Spectogram => {
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
    egui_dock::DockState::new(vec![Tab::Volume, Tab::Frequency, Tab::Spectogram])
}
