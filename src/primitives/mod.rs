// このファイルは、primitivesモジュールに含まれる他のファイルを宣言し、
// それらの中の公開アイテム（pub）を、このモジュールの外からも使えるようにします。

// 各プリミティブのモジュールを宣言
mod sphere;
mod plane;
mod infinite_cylinder;
mod infinite_cone;
mod axis_aligned_box;
mod csg;
mod lens;
mod wedge;
mod transform;

// 各モジュール内の公開アイテムを、primitives::* で使えるように再公開（re-export）する
pub use sphere::Sphere;
pub use plane::Plane;
pub use infinite_cylinder::InfiniteCylinder;
pub use infinite_cone::InfiniteCone;
pub use axis_aligned_box::AxisAlignedBox;
pub use csg::{CSGObject, CsgOperation};
pub use lens::Lens;
pub use wedge::Wedge;
pub use transform::Transform;