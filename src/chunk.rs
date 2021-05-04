use std::sync::{Arc, Mutex};

use cgmath::*;

use crate::{atlas::*, block::*, Config};

pub const CHUNK_Y_SIZE: usize = 200;
pub const CHUNK_Z_SIZE: usize = 16;
pub const CHUNK_X_SIZE: usize = 16;

pub const SEA_LEVEL: usize = CHUNK_Y_SIZE / 2;
pub const TOTAL_CHUNK_SIZE: usize = CHUNK_Y_SIZE * CHUNK_Z_SIZE * CHUNK_X_SIZE;

fn init_blocks_and_mesh(offset: [i32; 3]) -> (Vec<Vec<Vec<Arc<Mutex<Block>>>>>, Mesh) {
    let mut blocks = vec![vec![vec![Arc::new(Mutex::new(Block::new(BlockType::DIRT, [0, 0, 0], offset))); CHUNK_Z_SIZE]; CHUNK_X_SIZE]; CHUNK_Y_SIZE];

    let mut vertices: Vec<BlockVertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for y in 0..CHUNK_Y_SIZE {
        for z in 0..CHUNK_Z_SIZE {
            for x in 0..CHUNK_X_SIZE {
                let block_type = BlockType::DEBUG;
                let position = [x as i32, y as i32, z as i32];
                let block = Block::new(block_type, position, offset);

                let mut block_vertices = Vec::with_capacity(4 * 6);
                let mut block_indices = Vec::with_capacity(6 * 6);
                let mut face_counter: u16 = 0;
                for face in block.faces.iter() {
                    block_vertices.extend_from_slice(&face.vertices);
                    block_indices.extend_from_slice(&face.get_indices(face_counter));
                    face_counter += 1;
                }

                vertices.extend(block_vertices);
                indices.extend(block_indices);

                blocks[y][x][z] = Arc::new(Mutex::new(block));
            }
        }
    }

    let num_elements = indices.len() as u32;

    return (
        blocks,
        Mesh {
            vertices,
            indices,
            num_elements,
        },
    );
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<BlockVertex>,
    pub indices: Vec<u16>,
    pub num_elements: u32,
}

pub type Blocks = Vec<Vec<Vec<Arc<Mutex<Block>>>>>;

#[derive(Default)]
pub struct ChunkArray {
    pub mesh_array: Vec<Arc<Mutex<Mesh>>>,
    pub offset_array: Vec<Arc<Mutex<[i32; 3]>>>,
    pub blocks_array: Vec<Arc<Mutex<Blocks>>>,

    _config: Config,
}

impl ChunkArray {
    pub fn new_chunk(&mut self, offset: [i32; 3]) -> &Self {
        let (blocks, mesh) = init_blocks_and_mesh(offset);
        self.mesh_array.push(Arc::new(Mutex::new(mesh)));
        self.blocks_array.push(Arc::new(Mutex::new(blocks)));
        self.offset_array.push(Arc::new(Mutex::new(offset)));
        return self;
    }

    pub fn pos_in_chunk_bounds(pos: Vector3<i32>) -> bool {
        if pos.x >= 0 && pos.y >= 0 && pos.z >= 0 {
            if pos.x < CHUNK_X_SIZE as i32 && pos.y < CHUNK_Y_SIZE as i32 && pos.z < CHUNK_Z_SIZE as i32 {
                return true;
            }
        }
        return false;
    }

    pub fn change_block(&mut self, chunk_index: usize, position: [usize; 3], new_material_type: BlockType) {
        let x = position[0];
        let y = position[1];
        let z = position[2];
        self.blocks_array[chunk_index].lock().unwrap()[y][x][z]
            .lock()
            .unwrap()
            .update(new_material_type, self.offset_array[chunk_index].lock().unwrap().clone());
    }

    pub fn get_block(&self, chunk_index: usize, position: [i32; 3]) -> Block {
        let x = position[0] as usize;
        let y = position[1] as usize;
        let z = position[2] as usize;
        let block = self.blocks_array[chunk_index].lock().unwrap()[y][x][z].lock().unwrap().clone();
        return block;
    }
}
