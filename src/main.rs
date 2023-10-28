use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_pixel_buffer::prelude::*;

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

    let mut grid = Grid {
        cells: vec![0.; (SIMULATION_WIDTH * SIMULATION_HEIGHT * NUM_INDEX) as usize],
        sources: vec![Source::new(
            array_pos(SIMULATION_WIDTH / 2, SIMULATION_WIDTH / 2, 0),
            0.0,
            5.0,
        )],
        walls: vec![],
    };

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
struct Grid {
    /// full grid: [cur_bottom, cur_left, cur_top, cur_right, next_bottom, next_left, next_top, next_right, pressure]
    cells: Vec<f32>,
    /// A list of sources (all sin for now), containing the indices of the corresponding cells (index, phase, frequency)
    sources: Vec<Source>,
    /// A list of walls, containing the indices of the corresponding cells
    walls: Vec<usize>,
}

#[derive(Debug)]
struct Source {
    index: usize,
    phase: f32,
    frequency: f32,
}

impl Source {
    fn new(index: usize, phase: f32, frequency: f32) -> Self {
        Self {
            index,
            phase,
            frequency,
        }
    }
}

impl Grid {
    fn update_grid(&mut self) {
        //TODO: parallelize?
        for i in 0..SIMULATION_WIDTH * SIMULATION_HEIGHT {
            let array_pos: usize = (i * NUM_INDEX) as usize;

            self.cells[array_pos] = self.cells[array_pos + 4];
            self.cells[array_pos + 1] = self.cells[array_pos + 5];
            self.cells[array_pos + 2] = self.cells[array_pos + 6];
            self.cells[array_pos + 3] = self.cells[array_pos + 7];

            //calculate pressure
            self.cells[array_pos + 8] = 0.5
                * (self.cells[array_pos]
                    + self.cells[array_pos + 1]
                    + self.cells[array_pos + 2]
                    + self.cells[array_pos + 3]);
        }
    }

    fn calc_grid(&mut self) {
        //TODO: parallelize?
        for x in 1..SIMULATION_WIDTH - 1 {
            for y in 1..SIMULATION_HEIGHT - 1 {
                self.calc(
                    array_pos(x, y, 0),
                    self.cells[array_pos(x, y + 1, 2)],
                    self.cells[array_pos(x - 1, y, 3)],
                    self.cells[array_pos(x, y - 1, 0)],
                    self.cells[array_pos(x + 1, y, 1)],
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
    ) {
        self.cells[coord_one_d + 4] = 0.5 * (-bottom_top + left_right + top_bottom + right_left);
        self.cells[coord_one_d + 5] = 0.5 * (bottom_top - left_right + top_bottom + right_left);
        self.cells[coord_one_d + 6] = 0.5 * (bottom_top + left_right - top_bottom + right_left);
        self.cells[coord_one_d + 7] = 0.5 * (bottom_top + left_right + top_bottom - right_left);
    }

    fn apply_sources(&mut self, time: f32) {
        for source in self.sources.iter() {
            let sin_calc = (2. * PI * source.frequency * (time - source.phase)).sin(); //maybe needs to be optimized
            self.cells[source.index + 4] = sin_calc;
            self.cells[source.index + 5] = sin_calc;
            self.cells[source.index + 6] = sin_calc;
            self.cells[source.index + 7] = sin_calc;
        }
    }

    fn apply_walls(&mut self) {
        for &wall_index in self.walls.iter() {
            let (x, y) = array_pos_rev(wall_index as u32);
            self.cells[wall_index + 4] = WALL_FAC * self.cells[array_pos(x, y + 1, 2)];
            self.cells[wall_index + 5] = WALL_FAC * self.cells[array_pos(x - 1, y, 3)];
            self.cells[wall_index + 6] = WALL_FAC * self.cells[array_pos(x, y - 1, 0)];
            self.cells[wall_index + 7] = WALL_FAC * self.cells[array_pos(x + 1, y, 1)];
        }
    }
}

fn array_pos(x: u32, y: u32, index: u32) -> usize {
    (y * SIMULATION_WIDTH * NUM_INDEX + x * NUM_INDEX + index) as usize
}

fn array_pos_rev(i: u32) -> (u32, u32) {
    let x = (i / 9) % SIMULATION_WIDTH;
    let y = i / 9 / SIMULATION_WIDTH;
    (x, y)
}

fn full_grid_update(mut grid: ResMut<Grid>, time: Res<Time>) {
    grid.calc_grid();

    grid.apply_sources(time.elapsed_seconds());
    grid.apply_walls();

    grid.update_grid();
}

fn draw_pixels(mut pb: QueryPixelBuffer, grid: Res<Grid>, gradient: Res<GradientResource>) {
    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = grid.cells[array_pos(coords.x, coords.y, 8)];
        let color = gradient.0.at((p + 0.5) as f64);
        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
            a: 255,
        }
    });

    for &wall_index in grid.walls.iter() {
        let (x, y) = array_pos_rev(wall_index as u32);
        //TODO: handle result
        let _ = frame.set(
            UVec2::new(x, y),
            Pixel {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        );
    }
}

fn screen_to_grid(x: f32, y: f32, screen_width: f32, screen_height: f32) -> Option<(u32, u32)> {
    let x = (x - (screen_width - (SIMULATION_WIDTH/PIXEL_SIZE) as f32) / 2.) as u32;
    let y = (y - (screen_height - (SIMULATION_HEIGHT/PIXEL_SIZE) as f32) / 2.) as u32;

    println!("x: {}, y: {}", x, y);

    if x >= SIMULATION_WIDTH || y >= SIMULATION_HEIGHT {
        return None;
    }

    Some((x, y))
}

fn mouse_button_input(
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut grid: ResMut<Grid>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) =
                screen_to_grid(position.x, position.y, window.width(), window.height())
            {
                grid.sources.push(Source::new(array_pos(x, y, 0), 0.0, 5.0));
            }
        }
    }
    if buttons.just_released(MouseButton::Left) {
        // Left Button was released
    }
    if buttons.pressed(MouseButton::Right) {
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            if let Some((x, y)) =
                screen_to_grid(position.x, position.y, window.width(), window.height())
            {
                //TODO: because of the brush size, the indices may be out of bounds
                //TODO: make bush size variable
                grid.walls.push(array_pos(x, y, 0));
                grid.walls.push(array_pos(x + 1, y, 0));
                grid.walls.push(array_pos(x - 1, y, 0));
                grid.walls.push(array_pos(x, y + 1, 0));
                grid.walls.push(array_pos(x, y - 1, 0));
                grid.walls.push(array_pos(x + 1, y + 1, 0));
                grid.walls.push(array_pos(x - 1, y - 1, 0));
                grid.walls.push(array_pos(x + 1, y - 1, 0));
                grid.walls.push(array_pos(x - 1, y + 1, 0));
            }
        }
    }
    // we can check multiple at once with `.any_*`
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        // Either the left or the right button was just pressed
    }
}
