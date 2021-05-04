use cgmath::*;
use winit::event::*;

use crate::{atlas::*, camera::*, ray_tracer::*, world::*};

pub struct Player {
    pos_ray: Ray,
    block_pos_in_view: Option<Vector3<i32>>,
    block_face_direction_in_view: Vector3<i32>,
    selected_block: BlockType,
}

pub const RAY_MAX_DISTANCE: f32 = 6.0;

impl Player {
    pub fn new(camera: &Camera) -> Self {
        Self {
            pos_ray: Ray {
                origin: camera.position.to_vec(),
                direction: camera.direction,
            },
            block_pos_in_view: None,
            block_face_direction_in_view: Vector3::new(0, 0, 0),
            selected_block: BlockType::DEBUG,
        }
    }

    pub fn update2(&mut self, camera: &Camera, world: &mut World) {
        self.pos_ray = Ray {
            origin: camera.position.to_vec(),
            direction: camera.direction,
        };

        let mut block_pos = None;
        if let Some(ray_collision) = ray_block(self.pos_ray, RAY_MAX_DISTANCE, world) {
            block_pos = Some(ray_collision.block_pos);
            self.block_face_direction_in_view = ray_collision.block_face_direction;
        }

        self.block_pos_in_view = block_pos;
    }

    pub fn input(&mut self, event: &DeviceEvent, queue: &wgpu::Queue, world: &mut World) {
        match event {
            DeviceEvent::Button {
                button: 0, // Left Mouse Button
                state,
            } => {
                if let ElementState::Pressed = state {
                    self.destroy_block(queue, world);
                }
            }
            DeviceEvent::Button {
                button: 1, // right Mouse Button
                state,
            } => {
                if let ElementState::Pressed = state {
                    self.place_block(queue, world);
                }
            }
            _ => {}
        }
    }

    fn destroy_block(&mut self, queue: &wgpu::Queue, world: &mut World) {
        if let Some(pos) = self.block_pos_in_view {
            world.set_block(pos, BlockType::AIR, queue);
        }
    }

    fn place_block(&mut self, queue: &wgpu::Queue, world: &mut World) {
        if let Some(pos) = self.block_pos_in_view {
            let pos = pos + self.block_face_direction_in_view;
            if world.block_is_air(pos) {
                world.set_block(pos, self.selected_block, queue);
            }
        }
    }
}
