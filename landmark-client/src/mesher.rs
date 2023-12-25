use shipyard::*;

use crate::{
    color::Color,
    game_map::{Chunk, ChunkCoords, ChunkTag, FaceDirection, GameMap, InnerChunkCoords},
    loader::ResourceDictionary,
    model::{MissingModel, ModelConstructor, UpdatedModel, Vertex},
    transform::Transform,
};

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

        self.indices.push(start_index);
        self.indices.push(start_index + 1);
        self.indices.push(start_index + 2);
        self.indices.push(start_index + 2);
        self.indices.push(start_index + 1);
        self.indices.push(start_index + 3);
    }
}

#[derive(Debug)]
pub struct ConstructedChunk {
    pub coords: ChunkCoords,
    pub model_constructor: ModelConstructor,
}

#[derive(Debug, Clone)]
pub struct MeshChunkRequest<'a> {
    pub requested_coords: ChunkCoords,
    pub requested_chunk: &'a Chunk,
    pub adjacent_chunks: Vec<Option<&'a Chunk>>,
}

pub fn chunk_mesher_sys(
    game_map: UniqueView<GameMap>,
    resource_dictionary: UniqueView<ResourceDictionary>,
    chunks: View<ChunkTag>,
    mut missing_models: ViewMut<MissingModel>,
    mut updated_models: ViewMut<UpdatedModel>,
) {
    let mut processed_chunks: Vec<(EntityId, ModelConstructor)> = Vec::new();

    for (id, (chunk, _)) in (&chunks, &missing_models).iter().with_id() {
        let requested_coords = chunk.coords;
        let requested_chunk = game_map.chunks.get(&requested_coords).unwrap();

        let mut adjacent_chunks = Vec::with_capacity(6);
        for face in 0..6 {
            let dir = FaceDirection::from(face);
            let offset = ChunkCoords::from(dir);

            adjacent_chunks.push(game_map.chunks.get(&(requested_coords + offset)));
        }

        let request = MeshChunkRequest {
            requested_coords,
            requested_chunk,
            adjacent_chunks,
        };

        let model_constructor = mesh_chunk(&request, &resource_dictionary);

        processed_chunks.push((id, model_constructor));
    }

    for (id, model_constructor) in processed_chunks.into_iter() {
        missing_models.delete(id);
        updated_models.add_component_unchecked(id, UpdatedModel(model_constructor))
    }
}

/// Stores visibility of each face of each block in a chunk.
type FaceVisibilityMap = Vec<[bool; 6]>;

fn generate_visibility_map(request: &MeshChunkRequest) -> FaceVisibilityMap {
    let mut visibility_map: FaceVisibilityMap = vec![[false; 6]; Chunk::BLOCKS_COUNT as usize];

    for z in 0..Chunk::SIZE {
        for y in 0..Chunk::SIZE {
            for x in 0..Chunk::SIZE {
                // TODO: This function should check transparency of adjacent blocks
                let coords = InnerChunkCoords::new(x, y, z);
                if request.requested_chunk.get_block(coords).is_none() {
                    continue;
                }

                for face in 0..6 {
                    let dir = FaceDirection::from(face);

                    // Default values
                    let mut checked_chunk: Option<&Chunk> = Some(request.requested_chunk);
                    let mut checked_coords = coords + dir.into();

                    // Edge cases when we need to check adjacent chunks
                    if dir.is_positive() {
                        if dir.is_x() && x == Chunk::SIZE - 1 {
                            checked_coords = InnerChunkCoords::new(0, y, z);
                            checked_chunk = request.adjacent_chunks[face];
                        } else if dir.is_y() && y == Chunk::SIZE - 1 {
                            checked_coords = InnerChunkCoords::new(x, 0, z);
                            checked_chunk = request.adjacent_chunks[face];
                        } else if dir.is_z() && z == Chunk::SIZE - 1 {
                            checked_coords = InnerChunkCoords::new(x, y, 0);
                            checked_chunk = request.adjacent_chunks[face];
                        }
                    }

                    if dir.is_negative() {
                        if dir.is_x() && x == 0 {
                            checked_coords = InnerChunkCoords::new(Chunk::SIZE - 1, y, z);
                            checked_chunk = request.adjacent_chunks[face];
                        } else if dir.is_y() && y == 0 {
                            checked_coords = InnerChunkCoords::new(x, Chunk::SIZE - 1, z);
                            checked_chunk = request.adjacent_chunks[face];
                        } else if dir.is_z() && z == 0 {
                            checked_coords = InnerChunkCoords::new(x, y, Chunk::SIZE - 1);
                            checked_chunk = request.adjacent_chunks[face];
                        }
                    }

                    if let Some(chunk) = checked_chunk {
                        if chunk.get_block(checked_coords).is_none() {
                            visibility_map[coords.as_idx()][face] = true;
                        }
                    }
                }
            }
        }
    }

    visibility_map
}

fn mesh_chunk(
    request: &MeshChunkRequest,
    resource_dictionary: &ResourceDictionary,
) -> ModelConstructor {
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

                if let Some(block) = request.requested_chunk.get_block(coords) {
                    for face in 0..6 {
                        if visibility_map[coords.as_idx()][face] {
                            let color = resource_dictionary.get_block_data_from_id(block).color;
                            model_constructor.add_block_face(coords, face.into(), color);
                        }
                    }
                }
            }
        }
    }

    model_constructor
}
