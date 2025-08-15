use glam::Vec3;

use crate::{
    CSGObject, CsgOperation, HitRecord, Hittable, Material, Plane, Ray, RenderableShape,
};
//ウェッジ
pub struct Wedge {
    pub csg_object: Box<dyn Hittable>,
}
// Wedge構造体の実装ブロックを追加
impl Wedge {
    pub fn new(size: Vec3, wedge_angle_rad: f32, material: Material) -> Self {
        let width = size.x;
        let height = size.y;
        let half_depth = size.z / 2.0;

        // --- 5枚の平面を定義 ---
        let p1 = Box::new(Plane {
            // 底面 (y >= 0)
            point: Vec3::ZERO,
            normal: Vec3::Y,
            material,
        }) as Box<dyn Hittable>;

        let p2 = Box::new(Plane {
            // 垂直面 (x >= 0)
            point: Vec3::ZERO,
            normal: Vec3::X,
            material,
        }) as Box<dyn Hittable>;

        // 傾斜面
        let angle_cos = wedge_angle_rad.cos();
        let angle_sin = wedge_angle_rad.sin();
        let p3 = Box::new(Plane {
            point: Vec3::ZERO,
            normal: Vec3::new(-angle_sin, angle_cos, 0.0), // 法線で傾きを表現
            material,
        }) as Box<dyn Hittable>;

        let p4 = Box::new(Plane {
            // 前面キャップ (z <= half_depth)
            point: Vec3::new(0.0, 0.0, half_depth),
            normal: Vec3::NEG_Z, // 法線を反転させることで、zが小さい側が「内側」になる
            material,
        }) as Box<dyn Hittable>;

        let p5 = Box::new(Plane {
            // 背面キャップ (z >= -half_depth)
            point: Vec3::new(0.0, 0.0, -half_depth),
            normal: Vec3::Z,
            material,
        }) as Box<dyn Hittable>;

        // --- CSGの積集合で5枚の平面を組み合わせる ---
        let csg1 = Box::new(CSGObject {
            left: p1,
            right: p2,
            operation: CsgOperation::Intersection,
            renderable_shape_override: None,
        });
        let csg2 = Box::new(CSGObject {
            left: csg1,
            right: p3,
            operation: CsgOperation::Intersection,
            renderable_shape_override: None,
        });
        let csg3 = Box::new(CSGObject {
            left: csg2,
            right: p4,
            operation: CsgOperation::Intersection,
            renderable_shape_override: None,
        });
        let final_wedge = Box::new(CSGObject {
            left: csg3,
            right: p5,
            operation: CsgOperation::Intersection,
            renderable_shape_override: None,
        });

        Wedge {
            csg_object: final_wedge,
        }
    }
}
// WedgeのためのHittable実装を追加
impl Hittable for Wedge {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        self.csg_object.intersect_all(ray, t_min, t_max)
    }

    fn get_renderable_shape(&self) -> Option<RenderableShape> {
        // This is a CSG object, so we don't render it directly for now.
        None
    }

    fn get_transform(&self) -> glam::Mat4 {
        glam::Mat4::IDENTITY
    }
}
