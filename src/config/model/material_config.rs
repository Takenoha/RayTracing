use serde::Deserialize;

use crate::Material;

#[derive(Deserialize, Clone, Copy)] // 材質はコピーするのでClone, Copyも
#[serde(tag = "type")]
pub enum MaterialConfig {
    Glass { ior: f32 },
    HalfMirror { reflectance: f32 },
    Mirror,
}

impl Into<Material> for MaterialConfig {
    fn into(self) -> Material {
        match self {
            MaterialConfig::Mirror => Material::Mirror,
            MaterialConfig::Glass { ior } => Material::Glass { ior },
            MaterialConfig::HalfMirror { reflectance } => Material::HalfMirror { reflectance },
        }
    }
}
