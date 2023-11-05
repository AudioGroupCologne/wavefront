// use bevy::prelude::*;
// use bevy_pixel_buffer::prelude::*;
// use tlm_rs::components::{GradientResource, Source};
use tlm_rs::constants::*;
use tlm_rs::grid::{apply_system, calc_system, update_system, Grid};
// use tlm_rs::input::mouse_button_input;
// use tlm_rs::render::draw_pixels;

// fn main() {
//     let size: PixelBufferSize = PixelBufferSize {
//         size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
//         pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
//     };

//     let grid = Grid::default();

//     let gradient = GradientResource::with_custom();

//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_plugin(PixelBufferPlugin)
//         .insert_resource(grid)
//         .insert_resource(gradient)
//         .add_startup_systems((pixel_buffer_setup(size), Source::spawn_initial_sources))
//         .add_systems((bevy::window::close_on_esc, mouse_button_input))
//         .add_systems((calc_system, apply_system, update_system, draw_pixels).chain())
//         .run();
// }

use bevy::{core::Pod, prelude::*, reflect::TypeUuid};
use bevy_app_compute::prelude::*;
use bevy_pixel_buffer::prelude::{
    pixel_buffer_setup, GetFrame, Pixel, PixelBufferPlugin, PixelBufferSize, QueryPixelBuffer,
};
use bytemuck::Zeroable;

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct CalcShader;

impl ComputeShader for CalcShader {
    fn shader() -> ShaderRef {
        "shaders/calc.wgsl".into()
    }
}

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a8"]
struct SourceShader;

impl ComputeShader for SourceShader {
    fn shader() -> ShaderRef {
        "shaders/source.wgsl".into()
    }
}

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a9"]
struct UpdateShader;

impl ComputeShader for UpdateShader {
    fn shader() -> ShaderRef {
        "shaders/update.wgsl".into()
    }
}

#[derive(Resource)]
struct SimpleComputeWorker;

impl ComputeWorker for SimpleComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let worker = AppComputeWorkerBuilder::new(world)
            .add_staging(
                "grid",
                &vec![0.; (SIMULATION_HEIGHT * SIMULATION_WIDTH * NUM_INDEX) as usize],
            )
            .add_staging(
                "sources",
                &vec![Source {
                    index: Grid::coords_to_index(250, 250, 0) as u32,
                    value: 0.,
                }],
            )
            .add_uniform("SIMULATION_WIDTH", &SIMULATION_WIDTH)
            .add_uniform("SIMULATION_HEIGHT", &SIMULATION_HEIGHT)
            .add_uniform("NUM_INDEX", &NUM_INDEX)
            .add_pass::<CalcShader>(
                [SIMULATION_WIDTH - 2, SIMULATION_HEIGHT - 2, 1],
                &["SIMULATION_WIDTH", "NUM_INDEX", "grid"],
            )
            .add_pass::<SourceShader>([1, 1, 1], &["grid", "sources"])
            .add_pass::<UpdateShader>(
                [SIMULATION_WIDTH, SIMULATION_HEIGHT, 1],
                &["NUM_INDEX", "grid"],
            )
            .build();

        worker
    }
}

#[derive(ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct Source {
    index: u32,
    value: f32,
}

fn main() {
    let size: PixelBufferSize = PixelBufferSize {
        size: UVec2::new(SIMULATION_WIDTH, SIMULATION_HEIGHT),
        pixel_size: UVec2::new(PIXEL_SIZE, PIXEL_SIZE),
    };
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AppComputePlugin)
        .add_plugin(AppComputeWorkerPlugin::<SimpleComputeWorker>::default())
        .add_plugin(PixelBufferPlugin)
        .add_startup_system(pixel_buffer_setup(size))
        .add_system(simple_draw)
        .run();
}

fn simple_draw(
    mut pb: QueryPixelBuffer,
    mut compute_worker: ResMut<AppComputeWorker<SimpleComputeWorker>>,
    time: Res<Time>,
) {
    if !compute_worker.ready() {
        return;
    };

    let result: Vec<f32> = compute_worker.read_vec("grid");

    //TODO: destructure vec<f32> into vec<Source>
    // let sources: Vec<f32> = compute_worker.read_vec("sources");
    // let destructured_sources: Vec<Source> = sources
    //     .iter()
    //     .map(|x| Source {
    //         index: 0,
    //         value: *x,
    //     })
    //     .collect();
    // why would we need to read the sources?

    // compute_worker.write_slice::<f32>("sources", &[2., 3., 4., 5.]);

    compute_worker.write_slice::<f32>(
        "sources",
        &[
            Grid::coords_to_index(10, 10, 0) as f32,
            (10. * time.elapsed_seconds()).sin(),
        ],
    );
    // println!("{}", Grid::coords_to_index(10, 10, 0) as f32);
    // println!("{}", result[Grid::coords_to_index(10, 10, 4)]);

    let mut frame = pb.frame();
    frame.per_pixel_par(|coords, _| {
        let p = result[Grid::coords_to_index(coords.x, coords.y, 8)];
        Pixel {
            r: (p * 200.) as u8,
            g: (p * 200.) as u8,
            b: (p * 200.) as u8,
            a: 255,
        }
    });
}
