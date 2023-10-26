use std::f64::consts::PI;

use bevy::prelude::*;
use bevy_pixel_buffer::prelude::*;
use na::{vector, Matrix4, Vector4};
use rayon::prelude::*;

extern crate nalgebra as na;

const SIMULATION_WIDTH: u32 = 700;
const SIMULATION_HEIGHT: u32 = 700;

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
    let size = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(1, 1),
    };

    let mut grid = Grid(vec![
        Node::default();
        (SIMULATION_WIDTH * SIMULATION_HEIGHT) as usize
    ]);
    grid.set(
        SIMULATION_WIDTH / 2,
        SIMULATION_HEIGHT / 2,
        Node {
            node_type: NodeType::Source,
            ..Default::default()
        },
    );

    // let source = grid.get_mut(SIMULATION_WIDTH / 2, SIMULATION_HEIGHT / 2);
    // source.current = vector![1., 0., 0., 0.];

    let gradient = GradientResource(colorgrad::magma());

    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugin))
        .insert_resource(grid)
        .insert_resource(gradient)
        .add_systems(Startup, pixel_buffer_setup(size))
        .add_systems(FixedUpdate, update_nodes_system)
        .add_systems(PostUpdate, draw_colors_system)
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
