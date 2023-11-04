// use bevy::prelude::*;
// use bevy_pixel_buffer::prelude::*;
// use tlm_rs::components::{GradientResource, Source};
// use tlm_rs::constants::*;
// use tlm_rs::grid::{apply_system, calc_system, update_system, Grid};
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

use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::ShaderRef};
use bevy_app_compute::prelude::*;

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct SimpleShader;

impl ComputeShader for SimpleShader {
    fn shader() -> ShaderRef {
        "shaders/simple.wgsl".into()
    }
}

#[derive(Resource)]
struct SimpleComputeWorker;

impl ComputeWorker for SimpleComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let worker = AppComputeWorkerBuilder::new(world)
            .add_staging("grid", &vec![0., 1., 2.])
            // .add_staging("source_pos", &vec![0, 1, 2])
            // .add_staging("wall_pos", &vec![0, 1, 2])
            // .add_pass::<SimpleShader>([3, 1, 1], &["grid", "source_pos", "wall_pos"])
            .add_pass::<SimpleShader>([3, 1, 1], &["grid"])
            .build();

        worker
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AppComputePlugin)
        .add_plugin(AppComputeWorkerPlugin::<SimpleComputeWorker>::default())
        .add_system(test)
        .run();
}

fn test(mut compute_worker: ResMut<AppComputeWorker<SimpleComputeWorker>>) {
    if !compute_worker.ready() {
        return;
    };

    let result: Vec<f32> = compute_worker.read_vec("grid");

    // compute_worker.write_slice::<f32>("values", &[2., 3., 4., 5.]);

    println!("{:?}", result)
}
