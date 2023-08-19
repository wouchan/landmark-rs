use std::sync::mpsc::{self, Receiver, Sender};

use shipyard::*;

use crate::{
    color::Color,
    game_map::{Chunk, ChunkCoords, InnerChunkCoords},
    model::{ModelConstructor, Vertex},
    transform::Transform,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceDirection {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl From<usize> for FaceDirection {
    fn from(value: usize) -> Self {
        match value {
            0 => FaceDirection::PosX,
            1 => FaceDirection::NegX,
            2 => FaceDirection::PosY,
            3 => FaceDirection::NegY,
            4 => FaceDirection::PosZ,
            5 => FaceDirection::NegZ,
            _ => {
                log::error!("Incorrect value passed as face direction: {value}, expected values from range 0 to 5");
                panic!();
            }
        }
    }
}

trait ModelConstructorChunkExt {
    fn add_block_face(&mut self, coords: InnerChunkCoords, face_dir: FaceDirection, color: Color);
}

impl ModelConstructorChunkExt for ModelConstructor {
    fn add_block_face(&mut self, coords: InnerChunkCoords, face_dir: FaceDirection, color: Color) {
        // 2-----3
        // |\    |
        // | \ B |
        // |  \  |
        // | A \ |
        // |    \|
        // 0-----1
        //
        // A: 0 1 2
        // B: 2 1 3

        // create face at the center of coordinate system facing positive Y and rotate it
        let a = glam::Vec3::new(-0.5, 0.5, -0.5);
        let b = glam::Vec3::new(0.5, 0.5, -0.5);
        let c = glam::Vec3::new(-0.5, 0.5, 0.5);
        let d = glam::Vec3::new(0.5, 0.5, 0.5);

        // pack points into vector to simplify rotations
        let mut points = vec![a, b, c, d];

        let rot = match face_dir {
            FaceDirection::PosX => {
                // rotate ccw along Y and ccw along Z
                glam::Quat::from_euler(
                    glam::EulerRot::XZY,
                    0.0,
                    -90f32.to_radians(),
                    -90f32.to_radians(),
                )
            }
            FaceDirection::NegX => {
                // rotate cw along Y and cw along Z
                glam::Quat::from_euler(
                    glam::EulerRot::XZY,
                    0.0,
                    90f32.to_radians(),
                    90f32.to_radians(),
                )
            }
            FaceDirection::PosY => {
                // already rotated correctly
                glam::Quat::IDENTITY
            }
            FaceDirection::NegY => {
                // rotate 180 along X
                glam::Quat::from_euler(glam::EulerRot::ZYX, 0.0, 0.0, 180f32.to_radians())
            }
            FaceDirection::PosZ => {
                // rotate 180 along Y and cw along X
                glam::Quat::from_euler(
                    glam::EulerRot::ZXY,
                    0.0,
                    90f32.to_radians(),
                    180f32.to_radians(),
                )
            }
            FaceDirection::NegZ => {
                // rotate ccw along X
                glam::Quat::from_euler(glam::EulerRot::ZYX, 0.0, 0.0, -90f32.to_radians())
            }
        };

        // rotate them to face correct direction
        points = points.into_iter().map(|p| rot * p).collect();

        // translate them to corrent position
        points = points
            .into_iter()
            .map(|p| p + coords.as_block_center())
            .collect();

        // produce vertices from the calculated points
        let mut vertices: Vec<Vertex> = points
            .into_iter()
            .map(|p| Vertex {
                position: p,
                color: color.into(),
            })
            .collect();

        // append vertices
        self.vertices.append(&mut vertices);

        // append indices
        let start_index = if let Some(last) = self.indices.last() {
            *last + 1
        } else {
            0
        };

        self.indices.push(start_index + 0);
        self.indices.push(start_index + 1);
        self.indices.push(start_index + 2);
        self.indices.push(start_index + 2);
        self.indices.push(start_index + 1);
        self.indices.push(start_index + 3);
    }
}

#[derive(Debug, Unique)]
pub struct MeshRequestsSender {
    pub chunks: Sender<MeshChunkRequest>,
}

impl MeshRequestsSender {
    pub fn init() -> (Self, Receiver<MeshChunkRequest>) {
        let chunks_channel: (Sender<MeshChunkRequest>, Receiver<MeshChunkRequest>) =
            mpsc::channel();

        (
            Self {
                chunks: chunks_channel.0,
            },
            chunks_channel.1,
        )
    }
}

#[derive(Debug)]
pub struct MeshChunkRequest {
    pub requested_coords: ChunkCoords,
    pub requested_chunk: Chunk,
    pub pos_x_adj: Option<Chunk>,
    pub neg_x_adj: Option<Chunk>,
    pub pos_y_adj: Option<Chunk>,
    pub neg_y_adj: Option<Chunk>,
    pub pos_z_adj: Option<Chunk>,
    pub neg_z_adj: Option<Chunk>,
}

#[derive(Debug)]
pub struct ConstructedChunk {
    pub coords: ChunkCoords,
    pub model_constructor: ModelConstructor,
}

pub fn chunk_mesher_loop(requests: Receiver<MeshChunkRequest>, output: Sender<ConstructedChunk>) {
    while let Ok(request) = requests.recv() {
        let model_constructor = mesh_chunk(&request);

        output
            .send(ConstructedChunk {
                coords: request.requested_coords,
                model_constructor,
            })
            .unwrap();
    }
}

/// Stores visibility of each face of each block in a chunk.
type FaceVisibilityMap = Vec<[bool; 6]>;

fn generate_visibility_map(request: &MeshChunkRequest) -> FaceVisibilityMap {
    let mut visibility_map: FaceVisibilityMap = vec![[false; 6]; Chunk::BLOCKS_COUNT];

    for z in 0..Chunk::SIZE {
        for y in 0..Chunk::SIZE {
            for x in 0..Chunk::SIZE {
                // TODO: This function should check transparency of adjacent blocks
                let coords = InnerChunkCoords::new(x, y, z);
                if let None = request.requested_chunk.get_block(coords) {
                    continue;
                }

                // PosX
                if x == Chunk::SIZE - 1 {
                    if let Some(ref adjacents) = request.pos_x_adj {
                        if let None = adjacents.get_block(InnerChunkCoords::new(0, y, z)) {
                            visibility_map[coords.as_idx()][FaceDirection::PosX as usize] = true;
                        }
                    }
                } else {
                    if let None =
                        request
                            .requested_chunk
                            .get_block(InnerChunkCoords::new(x + 1, y, z))
                    {
                        visibility_map[coords.as_idx()][FaceDirection::PosX as usize] = true;
                    }
                }

                // NegX
                if x == 0 {
                    if let Some(ref adjacents) = request.neg_x_adj {
                        if let None =
                            adjacents.get_block(InnerChunkCoords::new(Chunk::SIZE - 1, y, z))
                        {
                            visibility_map[coords.as_idx()][FaceDirection::NegX as usize] = true;
                        }
                    }
                } else {
                    if let None =
                        request
                            .requested_chunk
                            .get_block(InnerChunkCoords::new(x - 1, y, z))
                    {
                        visibility_map[coords.as_idx()][FaceDirection::NegX as usize] = true;
                    }
                }

                // PosY
                if y == Chunk::SIZE - 1 {
                    if let Some(ref adjacents) = request.pos_y_adj {
                        if let None = adjacents.get_block(InnerChunkCoords::new(x, 0, z)) {
                            visibility_map[coords.as_idx()][FaceDirection::PosY as usize] = true;
                        }
                    }
                } else {
                    if let None =
                        request
                            .requested_chunk
                            .get_block(InnerChunkCoords::new(x, y + 1, z))
                    {
                        visibility_map[coords.as_idx()][FaceDirection::PosY as usize] = true;
                    }
                }

                // NegY
                if y == 0 {
                    if let Some(ref adjacents) = request.neg_y_adj {
                        if let None =
                            adjacents.get_block(InnerChunkCoords::new(x, Chunk::SIZE - 1, z))
                        {
                            visibility_map[coords.as_idx()][FaceDirection::NegY as usize] = true;
                        }
                    }
                } else {
                    if let None =
                        request
                            .requested_chunk
                            .get_block(InnerChunkCoords::new(x, y - 1, z))
                    {
                        visibility_map[coords.as_idx()][FaceDirection::NegY as usize] = true;
                    }
                }

                // PosZ
                if z == Chunk::SIZE - 1 {
                    if let Some(ref adjacents) = request.pos_z_adj {
                        if let None = adjacents.get_block(InnerChunkCoords::new(x, y, 0)) {
                            visibility_map[coords.as_idx()][FaceDirection::PosZ as usize] = true;
                        }
                    }
                } else {
                    if let None =
                        request
                            .requested_chunk
                            .get_block(InnerChunkCoords::new(x, y, z + 1))
                    {
                        visibility_map[coords.as_idx()][FaceDirection::PosZ as usize] = true;
                    }
                }

                // NegZ
                if z == 0 {
                    if let Some(ref adjacents) = request.neg_z_adj {
                        if let None =
                            adjacents.get_block(InnerChunkCoords::new(x, y, Chunk::SIZE - 1))
                        {
                            visibility_map[coords.as_idx()][FaceDirection::NegZ as usize] = true;
                        }
                    }
                } else {
                    if let None =
                        request
                            .requested_chunk
                            .get_block(InnerChunkCoords::new(x, y, z - 1))
                    {
                        visibility_map[coords.as_idx()][FaceDirection::NegZ as usize] = true;
                    }
                }
            }
        }
    }

    visibility_map
}

fn mesh_chunk(request: &MeshChunkRequest) -> ModelConstructor {
    let mut model_constructor = ModelConstructor::new();

    model_constructor.transform = Transform {
        rotation: glam::Quat::IDENTITY,
        translation: request.requested_coords.as_translation(),
    };

    let visibility_map = generate_visibility_map(request);

    for z in 0..Chunk::SIZE {
        for y in 0..Chunk::SIZE {
            for x in 0..Chunk::SIZE {
                let coords = InnerChunkCoords::new(x, y, z);
                if let None = request.requested_chunk.get_block(coords) {
                    continue;
                }

                for face in 0..6 {
                    if visibility_map[coords.as_idx()][face] {
                        let color = if (x + z) % 2 == 0 {
                            Color::new(16, 200, 16)
                        } else {
                            Color::new(16, 164, 16)
                        };

                        model_constructor.add_block_face(coords, face.into(), color);
                    }
                }
            }
        }
    }

    model_constructor
}
