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
