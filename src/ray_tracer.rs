use cgmath::*;
use num::*;

use crate::world::*;

// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.42.3443&rep=rep1&type=pdf

fn init_tmax(s: Vector3<f32>, ds: Vector3<f32>) -> Vector3<f32> {
    let mut tmax = Vector3::new(0.0, 0.0, 0.0);
    for i in 0..3 {
        let t = if ds[i] > 0.0 { s[i].ceil() - s[i] } else { s[i] - s[i].floor() };
        tmax[i] = t / ds[i].abs();
    }

    return tmax;
}
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct RayCollision {
    pub block_pos: Vector3<i32>,
    pub block_face_direction: Vector3<i32>,
}

pub fn ray_block(ray: Ray, max_distance: f32, world: &mut World) -> Option<RayCollision> {
    let mut face_direction_vec: Vector3<i32> = Vector3::new(0, 0, 0);
    let mut p: Vector3<i32> = Vector3::new(ray.origin.x.floor() as i32, ray.origin.y.floor() as i32, ray.origin.z.floor() as i32);
    let dir = ray.direction;

    let step: Vector3<i32> = Vector3::new(signum(dir.x) as i32, signum(dir.y) as i32, signum(dir.z) as i32);

    let mut tmax = init_tmax(ray.origin, dir);
    let tdelta: Vector3<f32> = step.cast().unwrap();
    let tdelta = tdelta.div_element_wise(dir);

    let radius = max_distance / (dir.dot(dir).sqrt());

    loop {
        if world.is_hitting_block(p) {
            return Some(RayCollision {
                block_pos: p,
                block_face_direction: face_direction_vec,
            });
        }
        if tmax.x < tmax.y {
            if tmax.x < tmax.z {
                if tmax.x > radius {
                    return None;
                }

                p.x += step.x;
                tmax.x += tdelta.x;
                face_direction_vec = vec3(-step.x, 0, 0);
            } else {
                if tmax.z > radius {
                    return None;
                }

                p.z += step.z;
                tmax.z += tdelta.z;
                face_direction_vec = vec3(0, 0, -step.z);
            }
        } else {
            if tmax.y < tmax.z {
                if tmax.y > radius {
                    return None;
                }

                p.y += step.y;
                tmax.y += tdelta.y;
                face_direction_vec = vec3(0, -step.y, 0);
            } else {
                if tmax.z > radius {
                    return None;
                }

                p.z += step.z;
                tmax.z += tdelta.z;
                face_direction_vec = vec3(0, 0, -step.z);
            }
        }
    }
}
