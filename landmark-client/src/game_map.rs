use core::fmt;
use std::collections::HashMap;

use shipyard::*;

use crate::mesher::{MeshChunkRequest, MeshRequestsSender};

pub type BlockId = u32;

#[derive(Debug, Unique)]
pub struct GameMap {
    pub chunks: HashMap<ChunkCoords, Chunk>,
    /// Maps chunk coordinates to corespoding entitiy ID - these should remain the same even if chunk is offloaded.
    pub chunk_entity_map: HashMap<ChunkCoords, EntityId>,
}

impl GameMap {
    pub fn new_test(world: &mut World) -> Self {
        let mut chunks = HashMap::new();
        let mut chunk_entity_map = HashMap::new();

        for cz in -5..5 {
            for cx in -5..5 {
                let coords = ChunkCoords::new(cx, 0, cz);
                let mut chunk = Chunk::new();

                for bz in 0..32 {
                    for bx in 0..32 {
                        let mut max_y = if (cx + cz) % 2 == 0 { 3 } else { 2 };

                        if bx >= 3 && bx <= Chunk::SIZE - 3 {
                            if bz >= 3 && bz <= Chunk::SIZE - 3 {
                                max_y += 1;
                            }
                        }

                        for by in 0..max_y {
                            chunk.set_block(InnerChunkCoords::new(bx, by, bz), Some(0));
                        }
                    }
                }

                chunks.insert(coords, chunk);
                chunk_entity_map.insert(
                    coords,
                    world.add_entity((ChunkTag { coords }, MissingChunkModel)),
                );
            }
        }

        Self {
            chunks,
            chunk_entity_map,
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct ChunkTag {
    pub coords: ChunkCoords,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct MissingChunkModel;

#[derive(Debug, Clone)]
pub struct Chunk {
    blocks: Vec<Option<BlockId>>,
}

impl Chunk {
    pub const SIZE: usize = 32;
    pub const BLOCKS_COUNT: usize = Chunk::SIZE * Chunk::SIZE * Chunk::SIZE;

    pub fn new() -> Self {
        let blocks = vec![None; Chunk::BLOCKS_COUNT];

        Self { blocks }
    }

    pub fn get_block(&self, coords: InnerChunkCoords) -> Option<BlockId> {
        self.blocks[coords.as_idx()]
    }

    pub fn set_block(&mut self, coords: InnerChunkCoords, block: Option<BlockId>) {
        self.blocks[coords.as_idx()] = block;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoords {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn as_translation(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.x as f32 * Chunk::SIZE as f32,
            self.y as f32 * Chunk::SIZE as f32,
            self.z as f32 * Chunk::SIZE as f32,
        )
    }
}

impl fmt::Display for ChunkCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InnerChunkCoords {
    x: usize,
    y: usize,
    z: usize,
}

impl InnerChunkCoords {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        if x >= Chunk::SIZE || y >= Chunk::SIZE || z >= Chunk::SIZE {
            log::error!(
                "Inner chunk coords out of bounds: ({x}, {y}, {z}) with chunk size of {}",
                Chunk::SIZE
            );

            panic!();
        }

        Self { x, y, z }
    }

    pub fn as_idx(&self) -> usize {
        self.z * Chunk::SIZE * Chunk::SIZE + self.y * Chunk::SIZE + self.x
    }

    pub fn as_block_center(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.x as f32 + 0.5,
            self.y as f32 + 0.5,
            self.z as f32 + 0.5,
        )
    }
}

impl fmt::Display for InnerChunkCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

pub fn mesh_missing_chunks_sys(
    requests: NonSync<UniqueView<MeshRequestsSender>>,
    game_map: UniqueView<GameMap>,
    chunks: View<ChunkTag>,
    mut missing_models: ViewMut<MissingChunkModel>,
) {
    let mut processed_chunks: Vec<EntityId> = Vec::new();

    for (id, (chunk, _)) in (&chunks, &missing_models).iter().with_id() {
        processed_chunks.push(id);

        let requested_coords = chunk.coords;
        let requested_chunk = game_map.chunks.get(&requested_coords).unwrap().clone();

        // TODO: this segment could be simplified a bit
        let neg_x_adj = if let Some(adj) = game_map.chunks.get(&ChunkCoords::new(
            requested_coords.x - 1,
            requested_coords.y,
            requested_coords.z,
        )) {
            Some(adj.clone())
        } else {
            None
        };

        let pos_x_adj = if let Some(adj) = game_map.chunks.get(&ChunkCoords::new(
            requested_coords.x + 1,
            requested_coords.y,
            requested_coords.z,
        )) {
            Some(adj.clone())
        } else {
            None
        };

        let neg_y_adj = if let Some(adj) = game_map.chunks.get(&ChunkCoords::new(
            requested_coords.x,
            requested_coords.y - 1,
            requested_coords.z,
        )) {
            Some(adj.clone())
        } else {
            None
        };

        let pos_y_adj = if let Some(adj) = game_map.chunks.get(&ChunkCoords::new(
            requested_coords.x,
            requested_coords.y + 1,
            requested_coords.z,
        )) {
            Some(adj.clone())
        } else {
            None
        };

        let neg_z_adj = if let Some(adj) = game_map.chunks.get(&ChunkCoords::new(
            requested_coords.x,
            requested_coords.y,
            requested_coords.z - 1,
        )) {
            Some(adj.clone())
        } else {
            None
        };

        let pos_z_adj = if let Some(adj) = game_map.chunks.get(&ChunkCoords::new(
            requested_coords.x,
            requested_coords.y,
            requested_coords.z + 1,
        )) {
            Some(adj.clone())
        } else {
            None
        };

        let request = MeshChunkRequest {
            requested_coords,
            requested_chunk,
            neg_x_adj,
            pos_x_adj,
            neg_y_adj,
            pos_y_adj,
            neg_z_adj,
            pos_z_adj,
        };

        requests.chunks.send(request).unwrap();
    }

    for id in processed_chunks.into_iter() {
        missing_models.delete(id);
    }
}
