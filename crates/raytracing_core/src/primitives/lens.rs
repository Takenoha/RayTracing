use crate::{
    CSGObject, CsgOperation, HitRecord, Hittable, InfiniteCylinder, Material, Plane, Ray,
    ShapeType, Sphere,
};
use glam::{f32, Vec3};
//レンズプリミティブ
pub struct Lens {
    pub csg_object: Box<dyn Hittable>,
}
// Lens構造体の実装ブロックを追加
impl Lens {
    pub fn new(center_thickness: f32, diameter: f32, r1: f32, r2: f32, material: Material) -> Self {
        // --- レンズの形状をCSGで組み立てる ---

        // 1. 2つの球面を定義する
        // レンズの中心を原点(0,0,0)に、光軸をZ軸に沿って配置する
        let half_thickness = center_thickness / 2.0;

        // 第1面 (光がZの負方向から来るとして、z = -half_thickness に頂点)
        let s1 = if r1.is_finite() {
            let center1 = Vec3::new(0.0, 0.0, -half_thickness + r1);
            Box::new(Sphere {
                center: center1,
                radius: r1.abs(),
                material,
            }) as Box<dyn Hittable>
        } else {
            // 曲率半径が無限大なら、平面
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, -half_thickness),
                normal: Vec3::Z,
                material,
            }) as Box<dyn Hittable>
        };

        // 第2面 (z = +half_thickness に頂点)
        let s2 = if r2.is_finite() {
            let center2 = Vec3::new(0.0, 0.0, half_thickness + r2);
            Box::new(Sphere {
                center: center2,
                radius: r2.abs(),
                material,
            }) as Box<dyn Hittable>
        } else {
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, half_thickness),
                normal: Vec3::NEG_Z,
                material,
            }) as Box<dyn Hittable>
        };

        // 2つの球面の積集合をとる
        let infinite_lens = Box::new(CSGObject {
            left: s1,
            right: s2,
            operation: CsgOperation::Intersection,
        });

        // 2. レンズの直径を制限する円柱を定義
        let aperture_cylinder = Box::new(InfiniteCylinder {
            axis_point: Vec3::ZERO,
            axis_dir: Vec3::Z, // 光軸
            radius: diameter / 2.0,
            material, // 材質はダミー
        });

        // 3. 無限レンズと円柱の積集合をとって、最終的なレンズ形状を完成させる
        let final_lens = Box::new(CSGObject {
            left: infinite_lens,
            right: aperture_cylinder,
            operation: CsgOperation::Intersection,
        });

        Lens {
            csg_object: final_lens,
        }
    }
}
// LensのためのHittable実装を追加
impl Hittable for Lens {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        self.csg_object.intersect_all(ray, t_min, t_max)
    }

    fn shape(&self) -> ShapeType {
        ShapeType::Lens
    }
}
