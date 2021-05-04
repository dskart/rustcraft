use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use cgmath::*;
use wgpu::util::DeviceExt;
use winit::event::*;

use crate::{chunk::CHUNK_Z_SIZE, renderer::*, world::WORLD_SIZE};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, position: Point3<f32>, camera_matrix: Matrix4<f32>, projection_matrix: Matrix4<f32>) {
        self.view_position = position.to_homogeneous().into();
        self.view_proj = (projection_matrix * camera_matrix).into()
    }
}

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub direction: Vector3<f32>,

    pub projection: Projection,
    pub camera_controller: CameraController,

    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_bind_group: wgpu::BindGroup,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
}

impl Camera {
    fn calc_direction(yaw: Rad<f32>, pitch: Rad<f32>) -> Vector3<f32> {
        let direction = Vector3::new(pitch.cos() * yaw.cos(), pitch.sin(), pitch.cos() * yaw.sin());

        return direction.normalize();
    }
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(renderer: &Renderer, position: V, yaw: Y, pitch: P) -> Self {
        let projection = Projection::new(
            renderer.sc_desc.width,
            renderer.sc_desc.height,
            cgmath::Deg(45.0),
            0.1,
            (WORLD_SIZE * CHUNK_Z_SIZE) as f32,
        );
        let camera_controller = CameraController::new(20.0, 0.4);

        let uniforms = Uniforms::new();

        let uniform_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let mut camera = Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            direction: Vector3::new(0.0, 0.0, 0.0),

            projection,
            camera_controller,

            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            uniforms,
        };

        camera.update(&renderer.queue, Duration::from_secs(0));

        return camera;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position,
            Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin()).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.camera_controller.process_mouse(delta.0, delta.1);
            }
            _ => {}
        }
    }

    pub fn input_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(key),
                    ..
                },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            _ => false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.projection.resize(new_size.width, new_size.height)
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        self.update_camera_controller(dt);
        self.uniforms.update_view_proj(self.position, self.calc_matrix(), self.projection.calc_matrix());
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn update_camera_controller(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward * (self.camera_controller.amount_forward - self.camera_controller.amount_backward) * self.camera_controller.speed * dt;
        self.position += right * (self.camera_controller.amount_right - self.camera_controller.amount_left) * self.camera_controller.speed * dt;

        self.position.y += (self.camera_controller.amount_up - self.camera_controller.amount_down) * self.camera_controller.speed * dt;

        // Rotate
        self.yaw += Rad(self.camera_controller.rotate_horizontal) * self.camera_controller.sensitivity * dt;
        self.pitch += Rad(-self.camera_controller.rotate_vertical) * self.camera_controller.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.camera_controller.rotate_horizontal = 0.0;
        self.camera_controller.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Rad(FRAC_PI_2) {
            self.pitch = -Rad(FRAC_PI_2);
        } else if self.pitch > Rad(FRAC_PI_2) {
            self.pitch = Rad(FRAC_PI_2);
        }

        self.direction = Camera::calc_direction(self.yaw, self.pitch);
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }
}
