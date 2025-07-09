use glam::Vec3;
use serde::Deserialize;

use crate::{
    primitives::{
        AxisAlignedBox, CSGObject, InfiniteCone, InfiniteCylinder, Lens, Plane, Sphere, Wedge,
    },
    CsgOperation, Hittable, Material,
};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ShapeConfig {
    Sphere {
        radius: f32,
    },
    Box {
        size: [f32; 3],
    },
    Plane {
        normal: [f32; 3],
    },
    Cylinder {
        height: f32,
        radius: f32,
    },
    Cone {
        angle_deg: f32,
        height: f32,
    }, // 有限円錐の定義を追加
    Wedge {
        size: [f32; 3],
        angle_deg: f32,
    },
    Lens {
        thickness: f32,
        diameter: f32,
        r1: f32,
        r2: f32,
    },
    // CSG（Constructive Solid Geometry）オブジェクトの定義
    Union {
        a: Box<ShapeConfig>,
        b: Box<ShapeConfig>,
    },
    Intersection {
        a: Box<ShapeConfig>,
        b: Box<ShapeConfig>,
    },
    Difference {
        a: Box<ShapeConfig>,
        b: Box<ShapeConfig>,
    },
}

impl ShapeConfig {
    pub fn into_with(self, material: Material) -> Box<dyn Hittable> {
        match self {
            ShapeConfig::Sphere { radius } => Box::new(Sphere {
                center: Vec3::ZERO,
                radius,
                material,
            }),
            ShapeConfig::Box { size } => {
                let s = Vec3::from_array(size) / 2.0;
                Box::new(AxisAlignedBox {
                    min: -s,
                    max: s,
                    material,
                })
            }
            ShapeConfig::Plane { normal } => Box::new(Plane {
                point: Vec3::ZERO,
                normal: Vec3::from_array(normal),
                material,
            }),
            ShapeConfig::Cylinder { height, radius } => {
                let half_height = height / 2.0;
                let body = Box::new(InfiniteCylinder {
                    axis_point: Vec3::ZERO,
                    axis_dir: Vec3::Y,
                    radius,
                    material,
                });
                let cap_top = Box::new(Plane {
                    point: Vec3::new(0.0, half_height, 0.0),
                    normal: Vec3::NEG_Y,
                    material,
                });
                let cap_bottom = Box::new(Plane {
                    point: Vec3::new(0.0, -half_height, 0.0),
                    normal: Vec3::Y,
                    material,
                });
                let capped_cylinder = Box::new(CSGObject {
                    left: body,
                    right: cap_top,
                    operation: CsgOperation::Intersection,
                });
                Box::new(CSGObject {
                    left: capped_cylinder,
                    right: cap_bottom,
                    operation: CsgOperation::Intersection,
                })
            }
            ShapeConfig::Cone { angle_deg, height } => {
                let cone = Box::new(InfiniteCone::new(
                    Vec3::ZERO,
                    Vec3::Y,
                    angle_deg.to_radians(),
                    material,
                ));
                let cap = Box::new(Plane {
                    point: Vec3::new(0.0, height, 0.0),
                    normal: Vec3::NEG_Y,
                    material,
                });
                Box::new(CSGObject {
                    left: cone,
                    right: cap,
                    operation: CsgOperation::Intersection,
                })
            }
            ShapeConfig::Wedge { size, angle_deg } => Box::new(Wedge::new(
                Vec3::from_array(size),
                angle_deg.to_radians(),
                material,
            )),
            ShapeConfig::Lens {
                thickness,
                diameter,
                r1,
                r2,
            } => Box::new(Lens::new(thickness, diameter, r1, r2, material)),
            ShapeConfig::Union { a, b } => Box::new(CSGObject {
                left: a.into_with(material.clone()),
                right: b.into_with(material),
                operation: CsgOperation::Union,
            }),
            ShapeConfig::Intersection { a, b } => Box::new(CSGObject {
                left: a.into_with(material.clone()),
                right: b.into_with(material),
                operation: CsgOperation::Intersection,
            }),
            ShapeConfig::Difference { a, b } => Box::new(CSGObject {
                left: a.into_with(material.clone()),
                right: b.into_with(material),
                operation: CsgOperation::Difference,
            }),
        }
    }
}
