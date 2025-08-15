// このファイルは、primitivesモジュールに含まれる他のファイルを宣言し、
// それらの中の公開アイテム（pub）を、このモジュールの外からも使えるようにします。

// 各プリミティブのモジュールを宣言
mod axis_aligned_box;
mod csg;
mod infinite_cone;
mod infinite_cylinder;
mod lens;
mod plane;
mod sphere;
mod transform;
mod wedge;

// 各モジュール内の公開アイテムを、primitives::* で使えるように再公開（re-export）する
pub use axis_aligned_box::AxisAlignedBox;
pub use csg::CSGObject;
pub use infinite_cone::InfiniteCone;
pub use infinite_cylinder::InfiniteCylinder;
pub use lens::Lens;
pub use plane::Plane;
pub use sphere::Sphere;
pub use transform::Transform;
pub use wedge::Wedge;

use crate::HitRecord;
use crate::Ray;
use glam::{Mat4, Vec3};

#[derive(Clone)]
pub enum RenderableShape {
    Sphere {
        radius: f32,
    },
    Box {
        size: Vec3,
    },
    Plane {
        normal: Vec3,
    },
    Cylinder {
        height: f32,
        radius: f32,
    },
    Cone {
        height: f32,
        angle_deg: f32,
    },
    Wedge {
        size: Vec3,
        angle_deg: f32,
    },
}

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
    Mirror,
    Glass { ior: f32 },
    HalfMirror { reflectance: f32 },
}

pub trait Hittable: Sync + Send {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>>;
    fn get_renderable_shape(&self) -> Option<RenderableShape>;
    fn get_transform(&self) -> Mat4;
}
