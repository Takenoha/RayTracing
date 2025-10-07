use glam::{Mat4, Vec3};
use raytracing_core::{Hittable, Material, Transform};
use serde::Deserialize;

use crate::{
    material_config::MaterialConfig, shape_config::ShapeConfig, transform_config::TransformConfig,
};

#[derive(Deserialize, Clone, Debug)]
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
        // X, Y, Z 回転を組み合わせる（順序: Z -> Y -> X）
        let rx = Mat4::from_rotation_x(transform_config.rotation_x_deg.to_radians());
        let ry = Mat4::from_rotation_y(transform_config.rotation_y_deg.to_radians());
        let rz = Mat4::from_rotation_z(transform_config.rotation_z_deg.to_radians());
        let rotation = rz * ry * rx;
        let transform_matrix = translation * rotation;

        Box::new(Transform::new(primitive, transform_matrix))
    }
}
