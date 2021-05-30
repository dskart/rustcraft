use anyhow::*;
use cgmath::*;
use wgpu::util::DeviceExt;

use crate::{bitmap_font::*, renderer::*};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OverlayInfoVertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

unsafe impl bytemuck::Pod for OverlayInfoVertex {}
unsafe impl bytemuck::Zeroable for OverlayInfoVertex {}

impl Vertex for OverlayInfoVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<OverlayInfoVertex>() as wgpu::BufferAddress,
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

fn overlay_info_vertex(pos: [f32; 2], tc: [f32; 2]) -> OverlayInfoVertex {
    OverlayInfoVertex {
        pos: [pos[0], pos[1], 0.0, 1.0],
        tex_coord: [tc[0], tc[1]],
    }
}

fn create_vertices(bitmap_font: &BitmapFont, params: DisplayParameters) -> Result<Vec<OverlayInfoVertex>> {
    let mut vertices: Vec<OverlayInfoVertex> = Vec::new();

    let mut x = params.x;

    let x_aspect_ratio = (5.0 / bitmap_font.common_parameters.texture_width as f32) * params.x_scale;
    let y_aspect_ratio = (5.0 / bitmap_font.common_parameters.texture_height as f32) * params.y_scale;

    for c in params.display_text.chars() {
        let glyph = bitmap_font.get_glyph(&c)?;

        let left = x + glyph.x_offset as f32 * x_aspect_ratio;
        let right = left + glyph.width as f32 * x_aspect_ratio;
        let u_left = glyph.x as f32 / bitmap_font.common_parameters.texture_width as f32;
        let u_right = (glyph.x + glyph.width) as f32 / bitmap_font.common_parameters.texture_width as f32;

        let bottom = params.y + glyph.y_offset as f32 * y_aspect_ratio;
        let top = bottom + glyph.height as f32 * y_aspect_ratio;
        let v_top = glyph.y as f32 / bitmap_font.common_parameters.texture_height as f32;
        let v_bottom = (glyph.y + glyph.height) as f32 / bitmap_font.common_parameters.texture_height as f32;

        let char_vertices = [
            overlay_info_vertex([left, bottom], [u_left, v_bottom]),
            overlay_info_vertex([right, bottom], [u_right, v_bottom]),
            overlay_info_vertex([left, top], [u_left, v_top]),
            overlay_info_vertex([right, bottom], [u_right, v_bottom]),
            overlay_info_vertex([left, top], [u_left, v_top]),
            overlay_info_vertex([right, top], [u_right, v_top]),
        ]
        .to_vec();

        vertices.extend(char_vertices.iter());
        x += glyph.x_advance as f32 * x_aspect_ratio;
    }

    vertices.reverse();

    return Ok(vertices);
}

const OVERLAY_INFO_PIXEL_SIZE: f32 = 100.0;

pub struct OverlayInfo {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: usize,
    pub render_pipeline: wgpu::RenderPipeline,

    pub bitmap_font: BitmapFont,
    display_string: String,
    screen_height: u32,
    screen_width: u32,
}

impl OverlayInfo {
    pub fn new(renderer: &Renderer) -> Result<Self> {
        let bitmap_font = BitmapFont::new(&renderer.device, &renderer.queue)?;

        let display_string = "FPS=999999|X=999999|Y=999999|Z=999999".to_string();
        let default_param = DisplayParameters::new(display_string.clone(), -1.0, 0.9, 0.5, 0.5);

        let vertices = create_vertices(&bitmap_font, default_param)?;

        let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("OverlayInfo Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let num_vertices = vertices.len() as usize;

        let render_pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bitmap_font.texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(
            &renderer.device,
            &render_pipeline_layout,
            wgpu::PrimitiveTopology::TriangleStrip,
            renderer.sc_desc.format,
            wgpu::BlendState::ALPHA_BLENDING,
            &[<OverlayInfoVertex>::desc()],
            wgpu::include_spirv!("../assets/shaders/2d_texture.vert.spv"),
            wgpu::include_spirv!("../assets/shaders/2d_texture.frag.spv"),
            false,
        );

        Ok(Self {
            vertex_buffer,
            num_vertices,
            render_pipeline,

            bitmap_font,
            display_string,
            screen_height: 100,
            screen_width: 100,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.screen_height = new_size.height;
        self.screen_width = new_size.width;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, fps: u32, camera_position: Point3<f32>) -> Result<()> {
        self.display_string = format!(
            "FPS={}|X={}|Y={}|Z={}",
            fps, camera_position.x as i32, camera_position.y as i32, camera_position.z as i32
        );

        let x_scale = OVERLAY_INFO_PIXEL_SIZE / self.screen_width as f32;
        let y_scale = OVERLAY_INFO_PIXEL_SIZE / self.screen_height as f32;
        let y_pos = ((1.0 * self.screen_height as f32) - OVERLAY_INFO_PIXEL_SIZE) / self.screen_height as f32;

        let default_param = DisplayParameters::new(self.display_string.clone(), -1.0, y_pos, x_scale, y_scale);
        let vertices = create_vertices(&self.bitmap_font, default_param)?;
        self.num_vertices = vertices.len() as usize;

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        Ok(())
    }
}

impl Draw for OverlayInfo {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, _: &'a wgpu::BindGroup) -> Result<()> {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.bitmap_font.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.num_vertices as u32, 0..1);
        Ok(())
    }
}
