use std::{collections::HashMap, fs};

use shipyard::*;

use crate::{block::BlockData, game_map::BlockId};

#[derive(Debug, Unique)]
pub struct ResourceDictionary {
    blocks: HashMap<BlockId, BlockData>,
    block_names: HashMap<String, BlockId>,
}

#[allow(unused)]
impl ResourceDictionary {
    pub fn new() -> Self {
        let mut blocks = HashMap::new();
        let mut block_names = HashMap::new();

        let block_data = load_block_data();
        for (idx, block) in block_data.into_iter().enumerate() {
            block_names.insert(block.name.clone(), idx as u32);
            blocks.insert(idx as u32, block);
        }

        Self {
            blocks,
            block_names,
        }
    }

    pub fn get_block_id(&self, name: &str) -> BlockId {
        *self.block_names.get(name).unwrap_or_else(|| {
            panic!("Requested a block with name {name} but its definition is not present")
        })
    }

    pub fn get_block_data_from_name(&self, name: &str) -> BlockData {
        self.blocks.get(&self.get_block_id(name)).unwrap().clone()
    }

    pub fn get_block_data_from_id(&self, id: BlockId) -> BlockData {
        self.blocks
            .get(&id)
            .unwrap_or_else(|| {
                panic!("Requested a block with id {id} but its definition is not present")
            })
            .clone()
    }
}

pub fn load_block_data() -> Vec<BlockData> {
    let root = "res/blocks";
    let paths = fs::read_dir(root).unwrap_or_else(|_| panic!("Directory {root} not found"));

    let mut blocks = Vec::new();

    for file in paths {
        let path = file.unwrap().path();
        let content = fs::read_to_string(path.clone()).unwrap();

        let data: BlockData = ron::from_str(content.as_str())
            .unwrap_or_else(|e| panic!("Failed to parse file {}: {e}", path.display()));

        blocks.push(data);
    }

    blocks
}
