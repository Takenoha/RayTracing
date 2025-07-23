// このファイルは、primitivesモジュールに含まれる他のファイルを宣言し、
// それらの中の公開アイテム（pub）を、このモジュールの外からも使えるようにします。

// 各プリミティブのモジュールを宣言
mod render_axis_aligned_box;
mod render_csg;
mod render_infinite_cone;
mod render_infinite_cylinder;
mod render_lens;
mod render_plane;
mod render_sphere;
mod render_wedge;

// 各モジュール内の公開アイテムを、primitives::* で使えるように再公開（re-export）する
pub use render_axis_aligned_box::AxisAlignedBox;
pub use render_csg::CSGObject;
pub use render_infinite_cone::InfiniteCone;
pub use render_infinite_cylinder::InfiniteCylinder;
pub use render_lens::Lens;
pub use render_plane::Plane;
pub use render_sphere::Sphere;
pub use render_wedge::Wedge;

use raytracing_core::HitRecord;
use raytracing_core::Ray;
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

pub trait Hittable {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>>;
}
