use glam::{Mat4, Vec3};
use raytracing_core::{Hittable, Material, Transform};
use serde::Deserialize;

use crate::{
    material_config::MaterialConfig, shape_config::ShapeConfig, transform_config::TransformConfig,
};

#[derive(Deserialize, Clone)]
pub struct ObjectConfig {
    pub shape: ShapeConfig,
    pub material: MaterialConfig,
    pub transform: TransformConfig,
}

impl Into<Box<dyn Hittable>> for ObjectConfig {
    fn into(self) -> Box<dyn Hittable> {
        let material: Material = self.material.into();

        let primitive = self.shape.into_with(material);

        // Transformを適用
        let transform_config = self.transform;
        let translation = Mat4::from_translation(Vec3::from_array(transform_config.position));
        let rotation = Mat4::from_rotation_y(transform_config.rotation_y_deg.to_radians());
        let transform_matrix = translation * rotation;

        Box::new(Transform::new(primitive, transform_matrix))
    }
}
