use cgmath::Vector3;

use crate::{atlas::BlockType, chunk::*, renderer::*};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BlockVertex {
    pos: [f32; 3],
    texture_coordinates: [f32; 2],
}

unsafe impl bytemuck::Pod for BlockVertex {}
unsafe impl bytemuck::Zeroable for BlockVertex {}

impl Vertex for BlockVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub fn block_vertex(pos: [i8; 3], material_type: BlockType, texture_corners: [u32; 2], position: [i32; 3], face_direction: FaceDirection) -> BlockVertex {
    let tc = material_type.get_texture_coordinates(texture_corners, face_direction);
    BlockVertex {
        pos: [
            pos[0] as f32 + position[0] as f32,
            pos[1] as f32 + position[1] as f32,
            pos[2] as f32 + position[2] as f32,
        ],
        texture_coordinates: [tc[0] as f32, tc[1] as f32],
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FaceDirection {
    TOP,
    BOTTOM,
    RIGHT,
    LEFT,
    FRONT,
    BACK,
}

impl FaceDirection {
    pub fn to_vec(self) -> Vector3<i32> {
        match self {
            FaceDirection::TOP => Vector3::new(0, 1, 0),
            FaceDirection::BOTTOM => Vector3::new(0, -1, 0),
            FaceDirection::RIGHT => Vector3::new(1, 0, 0),
            FaceDirection::LEFT => Vector3::new(-1, 0, 0),
            FaceDirection::FRONT => Vector3::new(0, 0, 1),
            FaceDirection::BACK => Vector3::new(0, 0, -1),
        }
    }

    fn get_vertices(self, material_type: BlockType, position: [i32; 3]) -> [BlockVertex; 4] {
        match self {
            FaceDirection::TOP => [
                block_vertex([0, 1, 0], material_type, [0, 0], position, self),
                block_vertex([0, 1, 1], material_type, [0, 1], position, self),
                block_vertex([1, 1, 1], material_type, [1, 1], position, self),
                block_vertex([1, 1, 0], material_type, [1, 0], position, self),
            ],
            FaceDirection::BOTTOM => [
                block_vertex([0, 0, 1], material_type, [0, 0], position, self),
                block_vertex([0, 0, 0], material_type, [0, 1], position, self),
                block_vertex([1, 0, 0], material_type, [1, 1], position, self),
                block_vertex([1, 0, 1], material_type, [1, 0], position, self),
            ],
            FaceDirection::RIGHT => [
                block_vertex([1, 1, 1], material_type, [0, 0], position, self),
                block_vertex([1, 0, 1], material_type, [0, 1], position, self),
                block_vertex([1, 0, 0], material_type, [1, 1], position, self),
                block_vertex([1, 1, 0], material_type, [1, 0], position, self),
            ],
            FaceDirection::LEFT => [
                block_vertex([0, 1, 0], material_type, [0, 0], position, self),
                block_vertex([0, 0, 0], material_type, [0, 1], position, self),
                block_vertex([0, 0, 1], material_type, [1, 1], position, self),
                block_vertex([0, 1, 1], material_type, [1, 0], position, self),
            ],
            FaceDirection::FRONT => [
                block_vertex([0, 1, 1], material_type, [0, 0], position, self),
                block_vertex([0, 0, 1], material_type, [0, 1], position, self),
                block_vertex([1, 0, 1], material_type, [1, 1], position, self),
                block_vertex([1, 1, 1], material_type, [1, 0], position, self),
            ],
            FaceDirection::BACK => [
                block_vertex([1, 1, 0], material_type, [0, 0], position, self),
                block_vertex([1, 0, 0], material_type, [0, 1], position, self),
                block_vertex([0, 0, 0], material_type, [1, 1], position, self),
                block_vertex([0, 1, 0], material_type, [1, 0], position, self),
            ],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Face {
    pub vertices: [BlockVertex; 4],
    pub direction: FaceDirection,
}

impl Face {
    fn new(material_type: BlockType, face_direction: FaceDirection, position: [i32; 3]) -> Self {
        Self {
            vertices: face_direction.get_vertices(material_type, position),
            direction: face_direction,
        }
    }

    pub fn get_indices(&self, i: u16) -> [u16; 6] {
        let displacement = i * 4;
        [
            0 + displacement,
            1 + displacement,
            2 + displacement,
            2 + displacement,
            3 + displacement,
            0 + displacement,
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub faces: [Face; 6],
    pub position: [i32; 3],
    pub material_type: BlockType,
}

impl Block {
    pub fn new(material_type: BlockType, position: [i32; 3], chunk_offset: [i32; 3]) -> Self {
        let faces = Block::generate_faces(material_type, position, chunk_offset);

        Self {
            faces,
            position,
            material_type,
        }
    }

    fn generate_faces(material_type: BlockType, position: [i32; 3], chunk_offset: [i32; 3]) -> [Face; 6] {
        let world_pos = [
            position[0] + (chunk_offset[0] * CHUNK_X_SIZE as i32),
            position[1],
            position[2] + (chunk_offset[2] * CHUNK_Z_SIZE as i32),
        ];

        let top = Face::new(material_type, FaceDirection::TOP, world_pos);
        let bottom = Face::new(material_type, FaceDirection::BOTTOM, world_pos);
        let right = Face::new(material_type, FaceDirection::RIGHT, world_pos);
        let left = Face::new(material_type, FaceDirection::LEFT, world_pos);
        let front = Face::new(material_type, FaceDirection::FRONT, world_pos);
        let back = Face::new(material_type, FaceDirection::BACK, world_pos);

        [top, bottom, right, left, front, back]
    }

    pub fn update(&mut self, new_material_type: BlockType, offset: [i32; 3]) {
        self.material_type = new_material_type;
        self.faces = Block::generate_faces(new_material_type, self.position, offset);
    }
}
