use bevy::prelude::*;
use bevy_pixel_buffer::prelude::*;
use na::{vector, Matrix4, Vector4};

extern crate nalgebra as na;

const SIMULATION_WIDTH: u32 = 256;
const SIMULATION_HEIGHT: u32 = 256;

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

    fn set_next(&mut self, x: i32, y: i32, vec: Vector4<f64>) {
        self.0[(y * SIMULATION_WIDTH as i32 + x) as usize].next = vec;
    }
}

const SCATTERING_MATRIX: Matrix4<f64> = Matrix4::new(
    -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
);

#[derive(Resource)]
struct GradientResource(colorgrad::Gradient);

fn main() {
    let size = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(2, 2),
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

    // let source = grid.get_mut(32, 32);
    // source.current = vector![1., 0., 0., 0.];

    let gradient = GradientResource(colorgrad::magma());

    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugin))
        .insert_resource(grid)
        .insert_resource(gradient)
        .add_systems(Startup, pixel_buffer_setup(size))
        .add_systems(
            Update,
            (draw_colors_system, update_nodes_system, sine_system),
        )
        .run();
}

fn draw_colors_system(mut pb: QueryPixelBuffer, grid: Res<Grid>, gradient: Res<GradientResource>) {
    pb.frame().per_pixel(|coords, _| {
        let p = grid
            .get(coords.x as i32, coords.y as i32)
            .expect("grid matches canvas size")
            .get_pressure();
        let color = gradient.0.at(p);

        Pixel {
            r: (color.r * 255.) as u8,
            g: (color.g * 255.) as u8,
            b: (color.b * 255.) as u8,
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
    for i in 0..grid.0.len() {
        let (x, y) = index_to_coords(i);
        let left = grid.get(x - 1, y);
        let right = grid.get(x + 1, y);
        let top = grid.get(x, y - 1);
        let bottom = grid.get(x, y + 1);

        let node = grid.0[i].clone();
        match node.node_type {
            NodeType::Source => {
                let t = time.elapsed_seconds_f64();
                let source = grid.get_mut(SIMULATION_WIDTH / 2, SIMULATION_HEIGHT / 2);
                let sin = (10. * t).sin() * 2.;
                source.current = vector![sin, sin, sin, sin];
            }
            _ => grid.0[i].next = node.calc(left, right, top, bottom),
        }
    }

    grid.0.iter_mut().for_each(|node| {
        node.update();
    });
}

fn sine_system(time: Res<Time>, mut grid: ResMut<Grid>) {
    let t = time.elapsed_seconds_f64();
    let source = grid.get_mut(SIMULATION_WIDTH / 2, SIMULATION_HEIGHT / 2);
    let sin = (2. * t).sin();
    source.current = vector![sin, sin, sin, sin];
}
