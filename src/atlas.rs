use anyhow::*;

use crate::{block::*, texture::*};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum BlockType {
    DIRT,
    GRASS,
    ROCK,
    WATER,
    AIR,
    DEBUG,
}

impl BlockType {
    pub fn get_texture_coordinates(&self, texture_corner: [u32; 2], face_direction: FaceDirection) -> [f32; 2] {
        match self {
            BlockType::GRASS => match face_direction {
                FaceDirection::TOP => atlas_pos_to_coordinates([0.0, 0.0], texture_corner),
                FaceDirection::BOTTOM => atlas_pos_to_coordinates([2.0, 0.0], texture_corner),
                FaceDirection::RIGHT => atlas_pos_to_coordinates([1.0, 0.0], texture_corner),
                FaceDirection::LEFT => atlas_pos_to_coordinates([1.0, 0.0], texture_corner),
                FaceDirection::FRONT => atlas_pos_to_coordinates([1.0, 0.0], texture_corner),
                FaceDirection::BACK => atlas_pos_to_coordinates([1.0, 0.0], texture_corner),
            },
            BlockType::DIRT => atlas_pos_to_coordinates([2.0, 0.0], texture_corner),
            BlockType::ROCK => atlas_pos_to_coordinates([3.0, 0.0], texture_corner),
            BlockType::WATER => atlas_pos_to_coordinates([0.0, 15.0], texture_corner),
            BlockType::AIR => [0.0, 0.0],
            BlockType::DEBUG => atlas_pos_to_coordinates([15.0, 0.0], texture_corner),
        }
    }
}

const BLOCK_PIXEL_SIZE: f32 = 16.0;
const ATLAS_PIXEL_SIZE: f32 = 256.0;

fn atlas_pos_to_coordinates(atlas_pos: [f32; 2], texture_corner: [u32; 2]) -> [f32; 2] {
    let mut pixel_x = atlas_pos[0] * BLOCK_PIXEL_SIZE;
    let mut pixel_y = atlas_pos[1] * BLOCK_PIXEL_SIZE;

    if texture_corner[0] == 1 {
        pixel_x += 15.0;
    }

    if texture_corner[1] == 1 {
        pixel_y += 16.0;
    }

    return [pixel_x / ATLAS_PIXEL_SIZE, pixel_y / ATLAS_PIXEL_SIZE];
}

pub struct Atlas {
    pub diffuse_texture: Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl Atlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bytes = include_bytes!("../assets/images/blocks.png");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "blocks.png").unwrap();

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Ok(Self {
            diffuse_texture,
            diffuse_bind_group,
            texture_bind_group_layout,
        })
    }
}
