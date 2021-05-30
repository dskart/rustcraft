use anyhow::*;
use wgpu::util::DeviceExt;

use crate::{renderer::*, texture::*};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CrosshairVertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

unsafe impl bytemuck::Pod for CrosshairVertex {}
unsafe impl bytemuck::Zeroable for CrosshairVertex {}

impl Vertex for CrosshairVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<CrosshairVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

fn crosshair_vertex(pos: [f32; 2], tc: [u32; 2]) -> CrosshairVertex {
    CrosshairVertex {
        pos: [pos[0], pos[1], 0.0, 1.0],
        tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices(width: u32, height: u32) -> Vec<CrosshairVertex> {
    const CROSSHAIR_PIXEL_SIZE: f32 = 100.0;
    let x = CROSSHAIR_PIXEL_SIZE / width as f32;
    let y = CROSSHAIR_PIXEL_SIZE / height as f32;

    let vertex_data = [
        crosshair_vertex([-x, y], [0, 0]),
        crosshair_vertex([-x, -y], [0, 1]),
        crosshair_vertex([x, -y], [1, 1]),
        crosshair_vertex([-x, y], [0, 0]),
        crosshair_vertex([x, -y], [1, 1]),
        crosshair_vertex([x, y], [1, 0]),
    ]
    .to_vec();

    return vertex_data.to_vec();
}

pub struct Crosshair {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: usize,
    pub render_pipeline: wgpu::RenderPipeline,

    pub diffuse_texture: Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl Crosshair {
    pub fn new(renderer: &Renderer) -> Self {
        let texture_bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let diffuse_bytes = include_bytes!("../assets/images/crosshair.png");
        let diffuse_texture = Texture::from_bytes(&renderer.device, &renderer.queue, diffuse_bytes, "crosshair.png").unwrap();

        let diffuse_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let vertices = create_vertices(0, 0);

        let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("OverlayInfo Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let num_vertices = vertices.len() as usize;

        let render_pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(
            &renderer.device,
            &render_pipeline_layout,
            wgpu::PrimitiveTopology::TriangleList,
            renderer.sc_desc.format,
            wgpu::BlendState::ALPHA_BLENDING,
            &[<CrosshairVertex>::desc()],
            wgpu::include_spirv!("../assets/shaders/2d_texture.vert.spv"),
            wgpu::include_spirv!("../assets/shaders/2d_texture.frag.spv"),
            false,
        );

        Self {
            vertex_buffer,
            num_vertices,
            render_pipeline,

            diffuse_texture,
            diffuse_bind_group,
            texture_bind_group_layout,
        }
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, new_size: winit::dpi::PhysicalSize<u32>) {
        let height = new_size.height;
        let width = new_size.width;

        let vertices = create_vertices(width, height);
        self.num_vertices = vertices.len() as usize;

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }
}

impl Draw for Crosshair {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, _: &'a wgpu::BindGroup) -> Result<()> {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.num_vertices as u32, 0..1);
        Ok(())
    }
}
