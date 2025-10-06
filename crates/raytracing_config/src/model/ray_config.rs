use glam::Vec3;
use serde::Deserialize;

use raytracing_core::Ray;

#[derive(Deserialize, Debug)]
pub struct RayConfig {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub current_ior: f32,
}

impl Into<Ray> for RayConfig {
    fn into(self) -> Ray {
        Ray {
            origin: Vec3::from_array(self.origin),
            direction: Vec3::from_array(self.direction).normalize(),
            current_ior: self.current_ior,
        }
    }
}