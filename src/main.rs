use std::f64::consts::PI;

use bevy::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use bevy_pixels::prelude::*;
use na::{coordinates::X, vector, Matrix4, Vector4};
use rayon::{array, prelude::*};

extern crate nalgebra as na;

const SIMULATION_WIDTH: u32 = 700;
const SIMULATION_HEIGHT: u32 = 700;
const PIXEL_SIZE: u32 = 1;

#[derive(Resource)]
struct GradientResource(colorgrad::Gradient);

fn main() {
    // let size: PixelBufferSize = PixelBufferSize {
    //     size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
    //     pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    // };

    let mut grid = GridFloat(
        vec![0.; (SIMULATION_WIDTH * SIMULATION_HEIGHT * NUM_INDEX) as usize],
        vec![array_pos(50, 50, 0)],
        vec![],
    );

    for x in 1..SIMULATION_WIDTH {
        grid.2.push(array_pos(x, 25, 0));
    }

    for x in 1..SIMULATION_WIDTH / 10 {
        grid.1.push(array_pos(
            x * SIMULATION_WIDTH / 10,
            SIMULATION_WIDTH / 2,
            0,
        ));
    }

    let gradient = GradientResource(colorgrad::magma());

    App::new()
        // .add_plugins((DefaultPlugins, PixelBufferPlugin))
        .add_plugins((DefaultPlugins, PixelsPlugin::default()))
        .insert_resource(grid)
        .insert_resource(gradient)
        // .add_systems(Startup, pixel_buffer_setup(size))
        // .add_systems(FixedUpdate, update_nodes_system)
        // .add_systems(PostUpdate, draw_colors_system)
        // .add_systems(Update, (full_grid_update, draw_pixels))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Draw, draw)
        .run();
}

fn draw(mut wrapper_query: Query<&mut PixelsWrapper>) {
    let Ok(mut wrapper) = wrapper_query.get_single_mut() else {
        return;
    };

    let frame: &mut [u8] = wrapper.pixels.frame_mut();

    frame.copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff].repeat(frame.len() / 4));
}

const NUM_INDEX: u32 = 9; //cur_bottom cur_left cur_top cur_right next_bottom next_left next_top next_right pressure

#[derive(Debug, Resource)]
struct GridFloat(Vec<f32>, Vec<usize>, Vec<usize>); //full grid, sources (1d coords), walls (1d coords)

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
        let sin_calc = (time * 10.).sin();
        for &i in self.1.iter() {
            self.0[i + 4] = sin_calc;
            self.0[i + 5] = sin_calc;
            self.0[i + 6] = sin_calc;
            self.0[i + 7] = sin_calc;
        }
    }

    fn apply_walls(&mut self) -> () {
        for &i in self.2.iter() {
            let (x, y) = array_pos_rev(i as u32);
            self.0[i + 4] = self.0[array_pos(x, y + 1, 2)];
            self.0[i + 5] = self.0[array_pos(x - 1, y, 3)];
            self.0[i + 6] = self.0[array_pos(x, y - 1, 0)];
            self.0[i + 7] = self.0[array_pos(x + 1, y, 1)];
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

    // let sin_calc: f32 = (time.elapsed_seconds() * 10.).sin();
    // grid.apply_sources(sin_calc);
    grid.apply_sources(time.elapsed_seconds());
    grid.apply_walls();

    grid.update_grid();
}

// fn draw_pixels(mut pb: QueryPixelBuffer, grid: Res<GridFloat>, _gradient: Res<GradientResource>) {
//     let mut frame = pb.frame();
//     frame.per_pixel_par(|coords, _| {
//         let p = grid.0[array_pos(coords.x, coords.y, 8) as usize];
//         // let color = gradient.0.at(p);
//         Pixel {
//             r: (p * 255.) as u8,
//             g: (p * 255.) as u8,
//             b: (p * 255.) as u8,
//             a: 255,
//         }
//     });

//     for &i in grid.2.iter() {
//         let (x, y) = array_pos_rev(i as u32);
//         frame.set(
//             UVec2::new(x, y),
//             Pixel {
//                 r: (255) as u8,
//                 g: (0) as u8,
//                 b: (0) as u8,
//                 a: 255,
//             },
//         );
//     }
// }
