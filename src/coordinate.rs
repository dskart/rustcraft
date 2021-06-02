use anyhow::*;
use wgpu::util::DeviceExt;

use crate::renderer::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CoordinateVertex {
    pos: [f32; 3],
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for CoordinateVertex {}
unsafe impl bytemuck::Zeroable for CoordinateVertex {}

impl Vertex for CoordinateVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<CoordinateVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn coordinate_vertex(pos: [i8; 3], c: [u8; 3]) -> CoordinateVertex {
    CoordinateVertex {
        pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
        color: [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0],
    }
}

pub fn create_vertices() -> (Vec<CoordinateVertex>, Vec<u16>) {
    let vertex_data = [
        // z
        coordinate_vertex([0, 0, 0], [0, 0, 255]),
        coordinate_vertex([0, 0, 2], [0, 0, 255]),
        // x
        coordinate_vertex([0, 0, 0], [255, 0, 0]),
        coordinate_vertex([2, 0, 0], [255, 0, 0]),
        // y
        coordinate_vertex([0, 0, 0], [0, 255, 0]),
        coordinate_vertex([0, 2, 0], [0, 255, 0]),
    ];

    let index_data: &[u16] = &[
        0, 1, //z
        2, 3, //x
        4, 5, //y
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub struct Coordinate {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: usize,

    pub render_pipeline: wgpu::RenderPipeline,

    pub display_coordinates: bool,
}

impl Coordinate {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        display_coordinates: bool,
    ) -> Result<Self> {
        let (vertices, indices) = create_vertices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CoordinateVertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        let num_indices = indices.len() as usize;

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(
            device,
            &render_pipeline_layout,
            wgpu::PrimitiveTopology::LineList,
            sc_desc.format,
            wgpu::BlendState::REPLACE,
            &[<CoordinateVertex>::desc()],
            include_str!("../assets/shaders/line.wgsl"),
            false,
        );

        Ok(Self {
            vertex_buffer,
            index_buffer,
            num_indices,

            render_pipeline,

            display_coordinates,
        })
    }
}

impl Draw for Coordinate {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, uniforms: &'a wgpu::BindGroup) -> Result<()> {
        if self.display_coordinates {
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &uniforms, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices as u32, 0, 0..1);
        }
        Ok(())
    }
}
