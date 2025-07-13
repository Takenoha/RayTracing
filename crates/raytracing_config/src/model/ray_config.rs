use glam::Vec3;
use serde::Deserialize;

use raytracing_core::Ray;

#[derive(Deserialize)]
pub struct RayConfig {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Into<Ray> for RayConfig {
    fn into(self) -> Ray {
        Ray {
            origin: Vec3::from_array(self.origin),
            direction: Vec3::from_array(self.direction).normalize(),
            current_ior: 1.0,
        }
    }
}
