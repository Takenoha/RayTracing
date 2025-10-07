use crate::{
    AABB, CSGObject, CsgOperation, HitRecord, Hittable, InfiniteCylinder, Material, Plane, Ray,
    Sphere,
};
use glam::Vec3;
// レンズプリミティブ
pub struct Lens {
    pub csg_object: Box<dyn Hittable>,
}
// Lens構造体の実装ブロックを追加
impl Lens {
    pub fn new(center_thickness: f32, diameter: f32, r1: f32, r2: f32, material: Material) -> Self {
        // --- レンズの形状をCSGで組み立てる ---
        let mat_for_s1 = material.clone();
        let mat_for_s2 = material.clone();
        let mat_for_aperture = material.clone();

        // 数値境界対策
        const EPS: f32 = 1e-3;

        // レンズ中心を原点、光軸を +Z 方向とする
        let half_thickness = center_thickness / 2.0;

        // 頂点位置（surface vertex）を明示
        let v1_z = -half_thickness; // 前面の頂点 z
        let v2_z = half_thickness; // 背面の頂点 z

        // 球心は頂点とその外向き法線から計算する：
        // center = vertex - n_out * r
        // 前面の外向き法線 = -Z, 背面の外向き法線 = +Z
        let s1 = if r1.is_finite() {
            // 前面：外向き n_out = -Z -> center = v1 - n_out * r1 = v1 + Z * r1
            let center1 = Vec3::new(0.0, 0.0, v1_z) + Vec3::Z * r1;
            Box::new(Sphere {
                center: center1,
                radius: r1.abs(),
                material: mat_for_s1,
            }) as Box<dyn Hittable>
        } else {
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, v1_z),
                normal: -Vec3::Z,
                material: mat_for_s1,
            }) as Box<dyn Hittable>
        };

        let s2 = if r2.is_finite() {
            // 背面：外向き n_out = +Z -> center = v2 - n_out * r2 = v2 - Z * r2
            let center2 = Vec3::new(0.0, 0.0, v2_z) - Vec3::Z * r2;
            Box::new(Sphere {
                center: center2,
                radius: r2.abs(),
                material: mat_for_s2,
            }) as Box<dyn Hittable>
        } else {
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, v2_z),
                normal: Vec3::Z,
                material: mat_for_s2,
            }) as Box<dyn Hittable>
        };

        // 無限レンズ（球面のIntersection）
        let infinite_lens = Box::new(CSGObject {
            left: s1,
            right: s2,
            operation: CsgOperation::Intersection,
        });

        // 円柱（開口）: axis_dir を正規化し、半径にEPSを加えて境界の穴を回避
        let axis_dir = Vec3::Z.normalize_or_zero();
        // let aperture_cylinder = Box::new(InfiniteCylinder {
        //     axis_point: Vec3::ZERO,
        //     axis_dir,
        //     radius: (diameter / 2.0) + EPS,
        //     material: mat_for_aperture,
        // });

        let aperture_sphere = Box::new(Sphere {
            center: Vec3::ZERO,
            radius: (diameter / 2.0) + EPS,
            material: mat_for_aperture,
        });

        let final_lens = Box::new(CSGObject {
            left: infinite_lens,
            right: aperture_sphere,
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

    fn bounding_box(&self) -> Option<AABB> {
        self.csg_object.bounding_box()
    }

    fn clone_hittable(&self) -> Box<dyn Hittable> {
        Box::new(Lens {
            csg_object: self.csg_object.clone_hittable(),
        })
    }
}
