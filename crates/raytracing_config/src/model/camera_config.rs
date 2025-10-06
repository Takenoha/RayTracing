use glam::Vec3;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CameraConfig {
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    #[serde(default = "default_vup")]
    pub vup: Vec3,
    #[serde(default = "default_vfov")]
    pub vfov: f32,
}

fn default_vup() -> Vec3 {
    Vec3::Y
}

fn default_vfov() -> f32 {
    40.0
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            lookfrom: Vec3::new(0.0, 0.0, 25.0),
            lookat: Vec3::new(0.0, 0.0, 0.0),
            vup: default_vup(),
            vfov: default_vfov(),
        }
    }
}
