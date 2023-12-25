use core::fmt;
use std::{collections::HashMap, ops};

use shipyard::*;

use crate::model::MissingModel;

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

                        if (3..=Chunk::SIZE - 3).contains(&bx)
                            && (3..=Chunk::SIZE - 3).contains(&bz)
                        {
                            max_y += 1;
                        }

                        let block: u32 = (bx + bz) as u32 % 3;

                        for by in 0..max_y {
                            chunk.set_block(InnerChunkCoords::new(bx, by, bz), Some(block));
                        }
                    }
                }

                chunks.insert(coords, chunk);
                chunk_entity_map.insert(
                    coords,
                    world.add_entity((ChunkTag { coords }, MissingModel)),
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    blocks: Vec<Option<BlockId>>,
}

impl Chunk {
    pub const SIZE: i32 = 32;
    pub const BLOCKS_COUNT: i32 = Chunk::SIZE * Chunk::SIZE * Chunk::SIZE;

    pub fn new() -> Self {
        let blocks = vec![None; Chunk::BLOCKS_COUNT as usize];

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

impl ops::Add for ChunkCoords {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl From<FaceDirection> for ChunkCoords {
    fn from(value: FaceDirection) -> Self {
        let (mut x, mut y, mut z) = (0, 0, 0);

        match value {
            FaceDirection::PosX => x = 1,
            FaceDirection::NegX => x = -1,
            FaceDirection::PosY => y = 1,
            FaceDirection::NegY => y = -1,
            FaceDirection::PosZ => z = 1,
            FaceDirection::NegZ => z = -1,
        }

        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InnerChunkCoords {
    x: i32,
    y: i32,
    z: i32,
}

impl InnerChunkCoords {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn as_idx(&self) -> usize {
        let (x, y, z) = (self.x as usize, self.y as usize, self.z as usize);
        let chunk_size = Chunk::SIZE as usize;

        z * chunk_size * chunk_size + y * chunk_size + x
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

impl ops::Add for InnerChunkCoords {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl From<FaceDirection> for InnerChunkCoords {
    fn from(value: FaceDirection) -> Self {
        let (mut x, mut y, mut z) = (0, 0, 0);

        match value {
            FaceDirection::PosX => x = 1,
            FaceDirection::NegX => x = -1,
            FaceDirection::PosY => y = 1,
            FaceDirection::NegY => y = -1,
            FaceDirection::PosZ => z = 1,
            FaceDirection::NegZ => z = -1,
        }

        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceDirection {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl FaceDirection {
    pub fn is_positive(self) -> bool {
        match self {
            FaceDirection::PosX => true,
            FaceDirection::NegX => false,
            FaceDirection::PosY => true,
            FaceDirection::NegY => false,
            FaceDirection::PosZ => true,
            FaceDirection::NegZ => false,
        }
    }

    pub fn is_negative(self) -> bool {
        !self.is_positive()
    }

    pub fn is_x(self) -> bool {
        match self {
            FaceDirection::PosX => true,
            FaceDirection::NegX => true,
            FaceDirection::PosY => false,
            FaceDirection::NegY => false,
            FaceDirection::PosZ => false,
            FaceDirection::NegZ => false,
        }
    }

    pub fn is_y(self) -> bool {
        match self {
            FaceDirection::PosX => false,
            FaceDirection::NegX => false,
            FaceDirection::PosY => true,
            FaceDirection::NegY => true,
            FaceDirection::PosZ => false,
            FaceDirection::NegZ => false,
        }
    }

    pub fn is_z(self) -> bool {
        match self {
            FaceDirection::PosX => false,
            FaceDirection::NegX => false,
            FaceDirection::PosY => false,
            FaceDirection::NegY => false,
            FaceDirection::PosZ => true,
            FaceDirection::NegZ => true,
        }
    }
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
