use rayon::prelude::*;

use crate::{atlas::*, chunk::*, noise::*};

fn normalize_noise(val: f64) -> usize {
    return ((val + 1.0) * CHUNK_Y_SIZE as f64 / 2.0) as usize;
}

pub fn generate_chunk(blocks: &mut Blocks, offset: [i32; 3], seed: u32, flat_world: bool) {
    if flat_world {
        generate_flat_world(blocks, offset);
        return;
    }

    let noise_map = get_noise_map(offset, seed);
    (0..TOTAL_CHUNK_SIZE).into_par_iter().for_each(|i| {
        let z = i / (CHUNK_X_SIZE * CHUNK_Y_SIZE);
        let y = (i - z * CHUNK_X_SIZE * CHUNK_Y_SIZE) / CHUNK_X_SIZE;
        let x = i - CHUNK_X_SIZE * (y + CHUNK_Y_SIZE * z);

        let noise_height = noise_map.get_value(x, z);
        let new_height = normalize_noise(noise_height);

        let block_type = if y > new_height {
            if y <= SEA_LEVEL {
                BlockType::WATER
            } else {
                BlockType::AIR
            }
        } else if y == new_height {
            BlockType::GRASS
        } else if y == 0 {
            BlockType::ROCK
        } else {
            BlockType::DIRT
        };

        blocks[y][x][z].lock().unwrap().update(block_type, offset);
    });
}

pub fn generate_flat_world(blocks: &mut Blocks, offset: [i32; 3]) {
    (0..TOTAL_CHUNK_SIZE).into_par_iter().for_each(|i| {
        let z = i / (CHUNK_X_SIZE * CHUNK_Y_SIZE);
        let y = (i - z * CHUNK_X_SIZE * CHUNK_Y_SIZE) / CHUNK_X_SIZE;
        let x = i - CHUNK_X_SIZE * (y + CHUNK_Y_SIZE * z);

        let block_type = if y > SEA_LEVEL {
            BlockType::AIR
        } else if y == SEA_LEVEL {
            BlockType::GRASS
        } else if y == 0 {
            BlockType::ROCK
        } else {
            BlockType::DIRT
        };

        blocks[y][x][z].lock().unwrap().update(block_type, offset);
    });
}
