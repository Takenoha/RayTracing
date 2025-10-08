// このファイルは、primitivesモジュールに含まれる他のファイルを宣言し、
// それらの中の公開アイテム（pub）を、このモジュールの外からも使えるようにします。

// 各プリミティブのモジュールを宣言
mod aabb;
mod axis_aligned_box;
mod bvh;
mod csg;
mod hittable_list;
mod infinite_cone;
mod infinite_cylinder;
mod lens;
mod plane;
mod sphere;
mod transform;
mod wedge;

// 各モジュール内の公開アイテムを、primitives::* で使えるように再公開（re-export）する
pub use aabb::AABB;
pub use axis_aligned_box::AxisAlignedBox;
pub use bvh::BVHNode;
pub use csg::CSGObject;
pub use hittable_list::HittableList;
pub use infinite_cone::InfiniteCone;
pub use infinite_cylinder::InfiniteCylinder;
pub use lens::Lens;
pub use plane::Plane;
pub use sphere::Sphere;
pub use transform::Transform;
pub use wedge::Wedge;

use crate::HitRecord;
use crate::Ray;
use glam::Vec3;
// ブーリアン演算の種類
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CsgOperation {
    /// 和集合
    Union,
    /// 積集合
    Intersection,
    /// 差集合
    Difference,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    Metal { color: Vec3, fuzz: f32 },
    Glass { color: Vec3, ior: f32 },
    HalfMirror { reflectance: f32 },
    Diffuse { color: Vec3 },
    Light { color: Vec3 },
}

impl Material {
    pub fn emitted(&self) -> Vec3 {
        match *self {
            Material::Light { color } => color,
            _ => Vec3::ZERO,
        }
    }
}

pub trait Hittable: Sync + Send {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>>;
    fn bounding_box(&self) -> Option<AABB>;
    fn clone_hittable(&self) -> Box<dyn Hittable>;

    // Default implementation for light sampling PDF
    fn pdf_value(&self, _origin: Vec3, _direction: Vec3) -> f32 {
        0.0
    }

    // Default implementation for generating a random direction towards the object
    fn random(&self, _origin: Vec3) -> Vec3 {
        Vec3::X // Should not be called on non-lights, default to an arbitrary vector
    }
}
