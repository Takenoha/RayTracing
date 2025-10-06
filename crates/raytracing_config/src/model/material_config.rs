use glam::Vec3;
use serde::Deserialize;

use raytracing_core::Material;

#[derive(Deserialize, Clone, Copy)] // 材質はコピーするのでClone, Copyも
#[serde(tag = "type")]
pub enum MaterialConfig {
    Glass {
        #[serde(default = "default_glass_color")]
        color: Vec3,
        ior: f32,
    },
    HalfMirror { reflectance: f32 },
    Metal { color: Vec3, fuzz: f32 },
    Diffuse { color: Vec3 },
    Light { color: Vec3 },
}

fn default_glass_color() -> Vec3 {
    Vec3::ONE
}

impl From<MaterialConfig> for Material {
    fn from(config: MaterialConfig) -> Material {
        match config {
            MaterialConfig::Metal { color, fuzz } => Material::Metal { color, fuzz },
            MaterialConfig::Glass { color, ior } => Material::Glass { color, ior },
            MaterialConfig::HalfMirror { reflectance } => Material::HalfMirror { reflectance },
            MaterialConfig::Diffuse { color } => Material::Diffuse { color },
            MaterialConfig::Light { color } => Material::Light { color },
        }
    }
}
