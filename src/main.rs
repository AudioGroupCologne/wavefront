use std::f64::consts::PI;

use bevy::prelude::*;
use bevy_pixel_buffer::prelude::*;
use na::{coordinates::X, vector, Matrix4, Vector4};
use rayon::{array, prelude::*};

extern crate nalgebra as na;

const SIMULATION_WIDTH: u32 = 700;
const SIMULATION_HEIGHT: u32 = 700;
const PIXEL_SIZE: u32 = 1;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum NodeType {
    #[default]
    Normal,
    Wall,
    End,
    Source,
}

#[derive(Debug, Default, Clone)]
struct Node {
    current: Vector4<f64>,
    next: Vector4<f64>,
    node_type: NodeType,
}

impl Node {
    fn get_pressure(&self) -> f64 {
        // self.previous.row_sum().x / 2.
        (self.current.x + self.current.y + self.current.z + self.current.w) / 2.
    }

    fn update(&mut self) {
        self.current = self.next;
        // if self.current.x > 0. {
        //     info!("{:?}", self.current);
        //     info!("{:?}", self.next);
        // }
    }

    fn calc(
        &self,
        left: Option<&Node>,
        right: Option<&Node>,
        top: Option<&Node>,
        bottom: Option<&Node>,
    ) -> Vector4<f64> {
        match (left, right, top, bottom) {
            (Some(left), Some(right), Some(top), Some(bottom)) => {
                let v = vector![
                    bottom.current.x,
                    left.current.y,
                    top.current.z,
                    right.current.w,
                ];

                SCATTERING_MATRIX * v
            }
            _ => Vector4::zeros(),
        }
    }
}

#[derive(Debug, Resource)]
struct Grid(Vec<Node>);

impl Grid {
    fn get(&self, x: i32, y: i32) -> Option<&Node> {
        if x > SIMULATION_WIDTH as i32 - 1 || y > SIMULATION_HEIGHT as i32 - 1 || x < 0 || y < 0 {
            return None;
        }

        Some(&self.0[(y * SIMULATION_WIDTH as i32 + x) as usize])
    }

    fn get_mut(&mut self, x: u32, y: u32) -> &mut Node {
        &mut self.0[(y * SIMULATION_WIDTH + x) as usize]
    }

    fn set(&mut self, x: u32, y: u32, node: Node) {
        self.0[(y * SIMULATION_WIDTH + x) as usize] = node;
    }
}

#[rustfmt::skip]
const SCATTERING_MATRIX: Matrix4<f64> = Matrix4::new(
    -0.5, 0.5, 0.5, 0.5,
    0.5, -0.5, 0.5, 0.5,
    0.5, 0.5, -0.5, 0.5,
    0.5, 0.5, 0.5, -0.5,
);

#[derive(Resource)]
struct GradientResource(colorgrad::Gradient);

fn main() {
    let size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };

    // let mut grid = Grid(vec![
    //     Node::default();
    //     (SIMULATION_WIDTH * SIMULATION_HEIGHT) as usize
    // ]);
    // grid.set(
    //     SIMULATION_WIDTH / 2,
    //     SIMULATION_HEIGHT / 2,
    //     Node {
    //         node_type: NodeType::Source,
    //         ..Default::default()
    //     },
    // );

    // ACHTUNG BAUSTELLE !!!!

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

    // BAUSTELLE ENDE !!!!

    // let source = grid.get_mut(SIMULATION_WIDTH / 2, SIMULATION_HEIGHT / 2);
    // source.current = vector![1., 0., 0., 0.];

    let gradient = GradientResource(colorgrad::magma());

    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugin))
        .insert_resource(grid)
        .insert_resource(gradient)
        .add_systems(Startup, pixel_buffer_setup(size))
        // .add_systems(FixedUpdate, update_nodes_system)
        // .add_systems(PostUpdate, draw_colors_system)
        .add_systems(Update, (full_grid_update, draw_pixels))
        .run();
}

fn draw_colors_system(mut pb: QueryPixelBuffer, grid: Res<Grid>, _gradient: Res<GradientResource>) {
    //TODO: replace bevy_pixel_buffer with bevy_pixels for gpu rendering?
    pb.frame().per_pixel_par(|coords, _| {
        let p = grid
            .get(coords.x as i32, coords.y as i32)
            .expect("grid matches canvas size")
            .get_pressure();
        // let color = gradient.0.at(p);

        Pixel {
            r: (p * 255.) as u8,
            g: (p * 255.) as u8,
            b: (p * 255.) as u8,
            a: 255,
        }
    })
}

fn index_to_coords(index: usize) -> (i32, i32) {
    let x = index % SIMULATION_WIDTH as usize;
    let y = index / SIMULATION_WIDTH as usize;

    (x as i32, y as i32)
}

fn update_nodes_system(mut grid: ResMut<Grid>, time: Res<Time>) {
    //TODO: make this parallel (without borrowing issues on grid)
    (0..SIMULATION_HEIGHT as usize * SIMULATION_WIDTH as usize).for_each(|i| {
        let (x, y) = index_to_coords(i);
        //TODO: use u32 for coords
        let left = grid.get(x - 1, y);
        let right = grid.get(x + 1, y);
        let top = grid.get(x, y - 1);
        let bottom = grid.get(x, y + 1);

        // if x as u32 == SIMULATION_WIDTH / 2 && y as u32 == (SIMULATION_HEIGHT / 2) - 1 {
        //     info!("{:?}", grid.0[i])
        // }

        let node = grid.0[i].clone();
        match node.node_type {
            NodeType::Source => sin_source(time.elapsed_seconds_f64(), &mut grid),
            _ => grid.0[i].next = node.calc(left, right, top, bottom),
        }
    });

    grid.0.par_iter_mut().for_each(|node| {
        node.update();
    });
}

fn sin_source(t: f64, grid: &mut ResMut<Grid>) {
    let source = grid.get_mut(SIMULATION_WIDTH / 2, SIMULATION_HEIGHT / 2);
    let sin = (PI * 2. * t).cos() * 4.;
    source.current = vector![sin, sin, sin, sin];
}

// ACHTUNG BAUSTELLE !!!!

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

fn draw_pixels(mut pb: QueryPixelBuffer, grid: Res<GridFloat>, _gradient: Res<GradientResource>) {
    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = grid.0[array_pos(coords.x, coords.y, 8) as usize];
        // let color = gradient.0.at(p);
        Pixel {
            r: (p * 255.) as u8,
            g: (p * 255.) as u8,
            b: (p * 255.) as u8,
            a: 255,
        }
    });

    for &i in grid.2.iter() {
        let (x, y) = array_pos_rev(i as u32);
        frame.set(
            UVec2::new(x, y),
            Pixel {
                r: (255) as u8,
                g: (0) as u8,
                b: (0) as u8,
                a: 255,
            },
        );
    }
}
