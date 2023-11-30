use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
    utils::label,
    window::WindowPlugin,
};
use std::borrow::Cow;

use tlm_rs::constants::*;

const SIZE: (u32, u32) = (500, 500);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    // present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            TLMPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    //create Image to render to
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    commands.insert_resource(TLMImage(image));

    //add camera
    commands.spawn(Camera2dBundle::default());

    //create grid array
    commands.insert_resource(Grid {
        values: vec![0.; (SIMULATION_WIDTH * SIMULATION_HEIGHT) as usize], // * NUM_INDEX
    })
}

pub struct TLMPlugin;

impl Plugin for TLMPlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugins((ExtractResourcePlugin::<TLMImage>::default(),));
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("tlm", TLMNode::default());
        render_graph.add_node_edge("tlm", bevy::render::main_graph::node::CAMERA_DRIVER);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<TLMPipeline>();
    }
}

#[derive(Resource, Clone, Deref, ExtractResource)]
struct TLMImage(Handle<Image>);

#[derive(Resource)]
struct TLMBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    grid: Res<Grid>,
    pipeline: Res<TLMPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    tlm_image: Res<TLMImage>,
    render_device: Res<RenderDevice>,
) {
    let view = gpu_images.get(&tlm_image.0).unwrap();

    // let a = grid.as_bind_group(layout, render_device, images, fallback_image)

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.bind_group_layout,
        // &BindGroupEntries::single(&view.texture_view),
        &BindGroupEntries::with_indices(((0, &view.texture_view), (1, grid_buffer))),
    );
    commands.insert_resource(TLMBindGroup(bind_group));
}

#[derive(Resource)]
pub struct TLMPipeline {
    bind_group_layout: BindGroupLayout,
    calc_pipeline: CachedComputePipelineId,
}

impl FromWorld for TLMPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba8Unorm,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/tlm_calc.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let calc_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        TLMPipeline {
            bind_group_layout,
            calc_pipeline,
        }
    }
}

enum TLMState {
    Loading,
    Update,
}

struct TLMNode {
    state: TLMState,
}

impl Default for TLMNode {
    fn default() -> Self {
        Self {
            state: TLMState::Loading,
        }
    }
}

impl render_graph::Node for TLMNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<TLMPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            TLMState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.calc_pipeline)
                {
                    self.state = TLMState::Update;
                }
            }
            TLMState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<TLMBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<TLMPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            TLMState::Loading => {}
            TLMState::Update => {
                let calc_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.calc_pipeline)
                    .unwrap();
                pass.set_pipeline(calc_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

// -----------------------------------------
#[derive(Resource, AsBindGroup, Debug, Clone, ExtractResource)]
pub struct Grid {
    #[storage(1, visibility(compute))]
    values: Vec<f32>,
}
