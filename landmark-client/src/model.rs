use std::sync::mpsc::{self, Receiver, Sender};

use shipyard::*;
use wgpu::util::DeviceExt;

use crate::{
    color::RawColor,
    game_map::GameMap,
    mesher::ConstructedChunk,
    rendererer::Renderer,
    transform::{RawTransform, Transform},
};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: glam::Vec3,
    pub color: RawColor,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Debug)]
pub struct ModelConstructor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub transform: Transform,
}

impl ModelConstructor {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            transform: Transform::default(),
        }
    }
}

#[derive(Debug, Component)]
pub struct Model {
    _vertices: Vec<Vertex>,
    indices: Vec<u16>,
    _transform: Transform,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
}

impl Model {
    pub fn new(device: &wgpu::Device, model_constructor: ModelConstructor) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&model_constructor.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&model_constructor.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_data = vec![RawTransform::from(model_constructor.transform)];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            _vertices: model_constructor.vertices,
            indices: model_constructor.indices,
            _transform: model_constructor.transform,
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
}

#[derive(Debug, Unique)]
pub struct ConstructedModelsReceiver {
    pub chunks: Receiver<ConstructedChunk>,
}

impl ConstructedModelsReceiver {
    pub fn init() -> (Self, Sender<ConstructedChunk>) {
        let chunks_channel: (Sender<ConstructedChunk>, Receiver<ConstructedChunk>) =
            mpsc::channel();

        (
            Self {
                chunks: chunks_channel.1,
            },
            chunks_channel.0,
        )
    }
}

pub fn update_models_sys(
    requests: NonSync<UniqueView<ConstructedModelsReceiver>>,
    game_map: UniqueView<GameMap>,
    renderer: UniqueView<Renderer>,
    mut models: ViewMut<Model>,
) {
    while let Ok(constructed_chunk) = requests.chunks.try_recv() {
        let id = game_map
            .chunk_entity_map
            .get(&constructed_chunk.coords)
            .unwrap_or_else(|| { log::error!("Request to update chunk model with coordinates {} received, but chunk entity not present", constructed_chunk.coords); panic!(); }
        );

        let model = Model::new(&renderer.device, constructed_chunk.model_constructor);
        models.add_component_unchecked(*id, model);
    }
}
