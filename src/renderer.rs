use std::iter;

use anyhow::*;
use winit::window::Window;

use crate::{camera::*, coordinate::*, crosshair::*, overlay_info::*, texture::*, world::*};

pub trait Draw {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, uniforms: &'a wgpu::BindGroup) -> Result<()>;
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    primitive_topology: wgpu::PrimitiveTopology,
    color_format: wgpu::TextureFormat,
    color_blend: wgpu::BlendState,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    vs_src: wgpu::ShaderModuleDescriptor,
    fs_src: wgpu::ShaderModuleDescriptor,
    wireframe_mode: bool,
) -> wgpu::RenderPipeline {
    let vs_module = device.create_shader_module(&vs_src);
    let fs_module = device.create_shader_module(&fs_src);
    let mut primitive = wgpu::PrimitiveState::default();
    primitive.topology = primitive_topology;
    primitive.front_face = wgpu::FrontFace::Ccw;
    primitive.cull_mode = Some(wgpu::Face::Back);
    if wireframe_mode {
        primitive.polygon_mode = wgpu::PolygonMode::Line;
    }

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        primitive,
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: color_format,
                blend: Some(color_blend),
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: true,
        },
    })
}

#[allow(dead_code)]
pub struct Renderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub depth_texture: Texture,

    logger: slog::Logger,
}

impl Renderer {
    pub async fn new(logger: slog::Logger, window: &Window) -> Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::NON_FILL_POLYGON_MODE,
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_texture = Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        Ok(Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,

            depth_texture,
            logger,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    pub fn render(&mut self, camera: &Camera, world: &World, coordinate: &Coordinate, overlay_info: &OverlayInfo, crosshair: &Crosshair) -> Result<()> {
        let frame = self.swap_chain.get_current_frame()?.output;

        {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

                world.draw(&mut render_pass, &camera.uniform_bind_group)?;
                coordinate.draw(&mut render_pass, &camera.uniform_bind_group)?;
                overlay_info.draw(&mut render_pass, &camera.uniform_bind_group)?;
                crosshair.draw(&mut render_pass, &camera.uniform_bind_group)?;
            }
            self.queue.submit(iter::once(encoder.finish()));
        }

        Ok(())
    }
}
