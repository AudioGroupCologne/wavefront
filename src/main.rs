use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_pixel_buffer::prelude::*;
use rayon::array;

const SIMULATION_WIDTH: u32 = 700;
const SIMULATION_HEIGHT: u32 = 700;
const PIXEL_SIZE: u32 = 1;
const NUM_INDEX: u32 = 9; //cur_bottom cur_left cur_top cur_right next_bottom next_left next_top next_right pressure
const WALL_FAC: f32 = 1.;

fn main() {
    let size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };

    let mut grid = GridFloat(
        vec![0.; (SIMULATION_WIDTH * SIMULATION_HEIGHT * NUM_INDEX) as usize],
        vec![(
            array_pos(SIMULATION_WIDTH / 2, SIMULATION_WIDTH / 2, 0),
            0.0,
            5.0,
        )],
        vec![],
    );

    // for x in 1..SIMULATION_WIDTH {
    //     if x < SIMULATION_WIDTH / 2 - 5 || x > SIMULATION_WIDTH / 2 + 5 {
    //         grid.2.push(array_pos(x, 100, 0));
    //         grid.1.push((
    //             array_pos(x, SIMULATION_WIDTH / 2 + 5, 0),
    //             (x * 5) as f32,
    //             10.0,
    //         ))
    //     }
    // }

    let gradient = GradientResource(colorgrad::blues());

    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugin))
        .insert_resource(grid)
        .insert_resource(gradient)
        .add_systems(Startup, pixel_buffer_setup(size))
        .add_systems(Update, (bevy::window::close_on_esc, mouse_button_input))
        .add_systems(Update, (full_grid_update, draw_pixels))
        .run();
}

#[derive(Resource)]
struct GradientResource(colorgrad::Gradient);

#[derive(Debug, Resource)]
struct GridFloat(Vec<f32>, Vec<(usize, f32, f32)>, Vec<usize>); //full grid, sources (1d coords), walls (1d coords)

impl GridFloat {
    fn update_grid(&mut self) -> () {
        for i in 0..SIMULATION_WIDTH * SIMULATION_HEIGHT {
            let array_pos: usize = (i * NUM_INDEX) as usize;

            self.0[array_pos] = self.0[array_pos + 4];
            self.0[array_pos + 1] = self.0[array_pos + 5];
            self.0[array_pos + 2] = self.0[array_pos + 6];
            self.0[array_pos + 3] = self.0[array_pos + 7];

            //pressure
            self.0[array_pos + 8] = 0.5
                * (self.0[array_pos]
                    + self.0[array_pos + 1]
                    + self.0[array_pos + 2]
                    + self.0[array_pos + 3]);
        }
    }

    fn calc_grid(&mut self) -> () {
        for x in 1..SIMULATION_WIDTH - 1 {
            for y in 1..SIMULATION_HEIGHT - 1 {
                GridFloat::calc(
                    self,
                    array_pos(x, y, 0),
                    self.0[array_pos(x, y + 1, 2)],
                    self.0[array_pos(x - 1, y, 3)],
                    self.0[array_pos(x, y - 1, 0)],
                    self.0[array_pos(x + 1, y, 1)],
                );
            }
        }
    }

    fn calc(
        &mut self,
        coord_one_d: usize,
        bottom_top: f32,
        left_right: f32,
        top_bottom: f32,
        right_left: f32,
    ) -> () {
        self.0[coord_one_d + 4] = 0.5 * (-bottom_top + left_right + top_bottom + right_left);
        self.0[coord_one_d + 5] = 0.5 * (bottom_top - left_right + top_bottom + right_left);
        self.0[coord_one_d + 6] = 0.5 * (bottom_top + left_right - top_bottom + right_left);
        self.0[coord_one_d + 7] = 0.5 * (bottom_top + left_right + top_bottom - right_left);
    }

    fn apply_sources(&mut self, time: f32) -> () {
        for &i in self.1.iter() {
            let sin_calc = (2. * PI * i.2 * (time - i.1)).sin(); //maybe needs to be optimized
            self.0[i.0 + 4] = sin_calc;
            self.0[i.0 + 5] = sin_calc;
            self.0[i.0 + 6] = sin_calc;
            self.0[i.0 + 7] = sin_calc;
        }
    }

    fn apply_walls(&mut self) -> () {
        for &i in self.2.iter() {
            let (x, y) = array_pos_rev(i as u32);
            self.0[i + 4] = WALL_FAC * self.0[array_pos(x, y + 1, 2)];
            self.0[i + 5] = WALL_FAC * self.0[array_pos(x - 1, y, 3)];
            self.0[i + 6] = WALL_FAC * self.0[array_pos(x, y - 1, 0)];
            self.0[i + 7] = WALL_FAC * self.0[array_pos(x + 1, y, 1)];
        }
    }
}

fn array_pos(x: u32, y: u32, index: u32) -> usize {
    return (y * SIMULATION_WIDTH * NUM_INDEX + x * NUM_INDEX + index) as usize;
}

fn array_pos_rev(i: u32) -> (u32, u32) {
    let x = (i / 9) % SIMULATION_WIDTH;
    let y = i / 9 / SIMULATION_WIDTH;
    return (x, y);
}

fn full_grid_update(mut grid: ResMut<GridFloat>, time: Res<Time>) -> () {
    grid.calc_grid();

    grid.apply_sources(time.elapsed_seconds());
    grid.apply_walls();

    grid.update_grid();
}

fn draw_pixels(mut pb: QueryPixelBuffer, grid: Res<GridFloat>, gradient: Res<GradientResource>) {
    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = grid.0[array_pos(coords.x, coords.y, 8) as usize];
        let color = gradient.0.at((p + 0.5) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });

    for &i in grid.2.iter() {
        let (x, y) = array_pos_rev(i as u32);
        let _ = frame.set(
            UVec2::new(x, y),
            Pixel {
                r: (0) as u8,
                g: (0) as u8,
                b: (0) as u8,
                a: 255,
            },
        );
    }
}

// Bevy Stuff

fn mouse_button_input(
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut grid: ResMut<GridFloat>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = q_windows.single().cursor_position() {
            let x =
                (position.x - (q_windows.single().width() - SIMULATION_WIDTH as f32) / 2.) as u32;
            let y =
                (position.y - (q_windows.single().height() - SIMULATION_HEIGHT as f32) / 2.) as u32;

            // Check Bounds

            grid.1.push((array_pos(x as u32, y as u32, 0), 0.0, 5.0));
        }
    }
    if buttons.just_released(MouseButton::Left) {
        // Left Button was released
    }
    if buttons.pressed(MouseButton::Right) {
        // Right Button is being held down
    }
    // we can check multiple at once with `.any_*`
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        // Either the left or the right button was just pressed
    }
}
