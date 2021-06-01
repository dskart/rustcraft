use std::collections::VecDeque;

use anyhow::*;
use cgmath::*;
use core::panic;
use rand::Rng;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

use crate::{atlas::*, block::*, chunk::*, renderer::*, world_generation::*, Config};

pub const WORLD_SIZE: usize = 10;
pub const WORLD_ARRAY_SIZE: usize = WORLD_SIZE * WORLD_SIZE;

fn vec3_mod(a: Vector3<i32>, b: Vector3<i32>) -> Vector3<i32> {
    Vector3::new(a[0] % b[0], a[1] % b[1], a[2] % b[2])
}

pub struct ChunkBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl ChunkBuffer {
    pub fn new(device: &wgpu::Device, vertices: Vec<BlockVertex>, indices: Vec<u16>, num_elements: u32) -> Self {
        Self {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
            }),
            num_elements,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, mesh: &Mesh) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&mesh.indices));
        self.num_elements = mesh.num_elements;
    }
}

#[allow(dead_code)]
pub struct World {
    pub chunks: ChunkArray,
    chunk_indices: Arc<Mutex<[Option<usize>; WORLD_ARRAY_SIZE]>>,
    free_chunk_indices: Arc<Mutex<VecDeque<usize>>>,

    pub chunk_buffers: Vec<ChunkBuffer>,
    pub chunk_render_pipeline: wgpu::RenderPipeline,

    center_offset: Vector3<i32>,
    chunks_origin: Vector3<i32>,
    pub atlas: Atlas,
    world_seed: u32,
    logger: slog::Logger,
    config: Config,
}

impl World {
    pub fn new(logger: slog::Logger, config: Config, renderer: &Renderer, uniform_bind_group_layout: &wgpu::BindGroupLayout) -> Result<Self> {
        let atlas = Atlas::new(&renderer.device, &renderer.queue)?;

        let mut chunk_buffers = vec![];
        let mut chunks = ChunkArray::default();
        let chunk_indices: [Option<usize>; WORLD_ARRAY_SIZE] = [None; WORLD_ARRAY_SIZE];
        let mut free_chunk_indices = VecDeque::new();
        for x in 0..WORLD_ARRAY_SIZE {
            chunks.new_chunk([0, 0, 0]);
            if let Some(mesh) = chunks.mesh_array.last() {
                let mesh = mesh.lock().unwrap().clone();
                let chunk_buffer = ChunkBuffer::new(&renderer.device, mesh.vertices.clone(), mesh.indices.clone(), mesh.num_elements.clone());
                chunk_buffers.push(chunk_buffer);
                free_chunk_indices.push_back(x);
            }
        }

        let render_pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&atlas.texture_bind_group_layout, uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let chunk_render_pipeline = create_render_pipeline(
            &renderer.device,
            &render_pipeline_layout,
            wgpu::PrimitiveTopology::TriangleList,
            renderer.sc_desc.format,
            wgpu::BlendState::REPLACE,
            &[<BlockVertex>::desc()],
            include_str!("../assets/shaders/3d_texture.wgsl"),
            config.wireframe,
        );

        let center_offset = vec3(0, 0, 0);
        let chunks_origin = center_offset - Vector3::new(WORLD_SIZE as i32 / 2, 0, WORLD_SIZE as i32 / 2);

        let mut rng = rand::thread_rng();
        let world_seed = rng.gen::<u32>();

        let mut world = Self {
            chunks,
            chunk_indices: Arc::new(Mutex::new(chunk_indices)),
            free_chunk_indices: Arc::new(Mutex::new(free_chunk_indices)),

            chunk_buffers,
            chunk_render_pipeline,

            atlas,
            world_seed,
            chunks_origin,
            center_offset,
            logger,
            config,
        };

        world.load_empty_chunks(&renderer.queue);

        return Ok(world);
    }

    pub fn load_empty_chunks(&mut self, queue: &wgpu::Queue) {
        (0..WORLD_ARRAY_SIZE).into_par_iter().for_each(|i| {
            let chunk_index = self.chunk_indices.lock().unwrap()[i].clone();
            if let None = chunk_index {
                let new_index = self.free_chunk_indices.lock().unwrap().pop_front().clone();
                if let Some(new_index) = new_index {
                    let chunk_offset = self.get_chunk_offset(i);
                    if !self.chunk_in_bounds(chunk_offset) {
                        panic!("Error: Cannot load chunk")
                    }

                    *self.chunks.offset_array[new_index].lock().unwrap() = chunk_offset.into();
                    generate_chunk(
                        &mut self.chunks.blocks_array[new_index].lock().unwrap(),
                        chunk_offset.into(),
                        self.world_seed,
                        self.config.flat_world,
                    );

                    let mesh = self.compute_mesh(&self.chunks.blocks_array[new_index].lock().unwrap());
                    *self.chunks.mesh_array[new_index].lock().unwrap() = mesh;

                    self.chunk_indices.lock().unwrap()[i] = Some(new_index);
                } else {
                    panic!("Error: No free space for chunk")
                }
            }
        });

        (0..WORLD_ARRAY_SIZE).for_each(|i| {
            self.chunk_buffers[i].update(queue, &self.chunks.mesh_array[i].lock().unwrap());
        });
    }

    //TODO: clean this up ?
    pub fn compute_mesh(&self, blocks: &Blocks) -> Mesh {
        let mut vertices: Vec<BlockVertex> = Vec::with_capacity(4 * 6 * TOTAL_CHUNK_SIZE);
        let mut indices: Vec<u16> = Vec::with_capacity(6 * 6 * TOTAL_CHUNK_SIZE);
        for y in 0..CHUNK_Y_SIZE {
            for z in 0..CHUNK_Z_SIZE {
                for x in 0..CHUNK_X_SIZE {
                    let block = blocks[y][x][z].lock().unwrap();

                    match block.material_type {
                        BlockType::AIR => {}
                        _ => {
                            let block_pos = Vector3::new(block.position[0], block.position[1], block.position[2]);
                            let blocks = &blocks;

                            let mut block_vertices = Vec::with_capacity(4 * 6);
                            let mut block_indices = Vec::with_capacity(6 * 6);
                            let mut face_counter: u16 = 0;
                            for face in block.faces.iter() {
                                let direction = face.direction.to_vec();
                                let neighbour_pos = block_pos + direction;

                                let mut visible = false;
                                if ChunkArray::pos_in_chunk_bounds(neighbour_pos) {
                                    let neighbour = blocks[neighbour_pos.y as usize][neighbour_pos.x as usize][neighbour_pos.z as usize]
                                        .lock()
                                        .unwrap();

                                    if let BlockType::AIR = neighbour.material_type {
                                        visible = true;
                                    }
                                } else {
                                    visible = true;
                                }

                                if visible {
                                    block_vertices.extend_from_slice(&face.vertices);
                                    block_indices.extend_from_slice(&face.get_indices(face_counter));
                                    face_counter += 1;
                                }
                            }
                            let block_indices: Vec<u16> = block_indices.iter().map(|x| x + vertices.len() as u16).collect();
                            vertices.extend_from_slice(&block_vertices);
                            indices.extend_from_slice(&block_indices);
                        }
                    }
                }
            }
        }

        Mesh {
            num_elements: indices.len() as u32,
            vertices,
            indices,
        }
    }

    #[allow(dead_code)]
    fn block_pos_to_world_pos(block_pos: Vector3<i32>, chunk_offset: Vector3<i32>) -> Vector3<i32> {
        let world_pos = vec3(
            block_pos.x + (chunk_offset.x * CHUNK_X_SIZE as i32),
            block_pos.y,
            block_pos.z + (chunk_offset.z * CHUNK_Z_SIZE as i32),
        );

        return world_pos;
    }

    // world array index -> chunk offset
    fn get_chunk_offset(&self, i: usize) -> Vector3<i32> {
        return self.chunks_origin + Vector3::new(i as i32 % WORLD_SIZE as i32, 0, i as i32 / WORLD_SIZE as i32);
    }

    // chunk offset -> world array index
    fn get_chunk_world_index(&self, chunk_offset: Vector3<i32>) -> usize {
        let p = chunk_offset - self.chunks_origin;
        return p.z as usize * WORLD_SIZE + p.x as usize;
    }

    // chunk offset -> index for self.chunks
    fn get_chunk_index(&self, chunk_offset: Vector3<i32>) -> Option<usize> {
        let i = self.get_chunk_world_index(chunk_offset);
        return self.chunk_indices.lock().unwrap()[i].clone();
    }

    /// World position -> chunk offset.
    fn world_pos_to_chunk_offset(world_pos: Vector3<f32>) -> Vector3<i32> {
        return vec3(
            (world_pos.x / CHUNK_X_SIZE as f32).floor() as i32,
            0,
            (world_pos.z / CHUNK_Z_SIZE as f32).floor() as i32,
        );
    }

    fn chunk_in_bounds(&self, chunk_offset: Vector3<i32>) -> bool {
        let p = chunk_offset - self.chunks_origin;
        if p.x >= 0 && p.z >= 0 && p.x < WORLD_SIZE as i32 && p.z < WORLD_SIZE as i32 {
            return true;
        }
        return false;
    }

    /// World position to block position in chunk coordinates.
    fn world_pos_to_block_pos(world_pos: Vector3<i32>) -> Vector3<i32> {
        let chunk_size = vec3(CHUNK_X_SIZE as i32, CHUNK_Y_SIZE as i32, CHUNK_Z_SIZE as i32);
        let block_pos = vec3_mod(vec3_mod(world_pos, chunk_size) + chunk_size, chunk_size);
        return block_pos;
    }

    pub fn set_center(&mut self, queue: &wgpu::Queue, pos: Vector3<f32>) {
        let new_offset = World::world_pos_to_chunk_offset(pos);
        let new_origin = new_offset - Vector3::new(WORLD_SIZE as i32 / 2, 0, WORLD_SIZE as i32 / 2);
        if new_origin == self.chunks_origin {
            return;
        }

        self.center_offset = new_offset;
        self.chunks_origin = new_origin;

        let chunk_indices_copy = self.chunk_indices.lock().unwrap().clone();
        self.chunk_indices = Arc::new(Mutex::new([None; WORLD_ARRAY_SIZE]));
        for i in 0..WORLD_ARRAY_SIZE {
            match chunk_indices_copy[i] {
                Some(chunk_index) => {
                    let chunk_offset = self.chunks.offset_array[chunk_index].lock().unwrap().clone();
                    if self.chunk_in_bounds(chunk_offset.into()) {
                        let new_chunk_world_index = self.get_chunk_world_index(chunk_offset.into());
                        self.chunk_indices.lock().unwrap()[new_chunk_world_index] = Some(chunk_index);
                    } else {
                        self.free_chunk_indices.lock().unwrap().push_back(chunk_index);
                    }
                }
                None => {}
            }
        }

        self.load_empty_chunks(queue);
    }

    pub fn set_block(&mut self, world_pos: Vector3<i32>, block_type: BlockType, queue: &wgpu::Queue) {
        let world_pos_f = world_pos.cast().expect("Cannot convert vec3<i32> to vec3<f32>");
        let chunk_offset = World::world_pos_to_chunk_offset(world_pos_f);

        if let Some(chunk_index) = self.get_chunk_index(chunk_offset) {
            let block_pos = World::world_pos_to_block_pos(world_pos)
                .cast()
                .expect("Cannot convert vec3<i32> to vec3<usize>");
            self.chunks.change_block(chunk_index, block_pos.into(), block_type);

            let mesh = self.compute_mesh(&self.chunks.blocks_array[chunk_index].lock().unwrap());
            *self.chunks.mesh_array[chunk_index].lock().unwrap() = mesh;
            self.chunk_buffers[chunk_index].update(queue, &self.chunks.mesh_array[chunk_index].lock().unwrap());
        }
    }

    pub fn get_block(&self, world_pos: Vector3<i32>) -> Option<Block> {
        let world_pos_f = world_pos.cast().expect("Cannot convert vec3<i32> to vec3<f32>");
        let chunk_offset = World::world_pos_to_chunk_offset(world_pos_f);

        if world_pos_f.y >= 0.0 && world_pos_f.y < CHUNK_Y_SIZE as f32 && self.chunk_in_bounds(chunk_offset) {
            if let Some(chunk_index) = self.get_chunk_index(chunk_offset) {
                let block_pos = World::world_pos_to_block_pos(world_pos);
                return Some(self.chunks.get_block(chunk_index, block_pos.into()));
            }
            return None;
        }
        return None;
    }

    pub fn block_is_air(&mut self, world_pos: Vector3<i32>) -> bool {
        if let Some(block) = self.get_block(world_pos) {
            if let BlockType::AIR = block.material_type {
                return true;
            }
        }
        return false;
    }

    pub fn is_hitting_block(&mut self, pos: Vector3<i32>) -> bool {
        if let Some(block) = self.get_block(pos) {
            if let BlockType::AIR = block.material_type {
                return false;
            }
            return true;
        }
        return false;
    }
}

impl Draw for World {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, uniforms: &'a wgpu::BindGroup) -> Result<()> {
        for chunk_buffer in self.chunk_buffers.iter() {
            render_pass.set_pipeline(&self.chunk_render_pipeline);

            render_pass.set_bind_group(0, &self.atlas.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &uniforms, &[]);

            render_pass.set_vertex_buffer(0, chunk_buffer.vertex_buffer.slice(..));
            render_pass.set_index_buffer(chunk_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..chunk_buffer.num_elements as u32, 0, 0..1);
        }
        return Ok(());
    }
}
