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
        // 入力パラメータの検証とデバッグ出力
        if center_thickness <= 0.0 {
            eprintln!(
                "Warning: center_thickness should be positive (got {})",
                center_thickness
            );
        }

        let mat_for_s1 = material.clone();
        let mat_for_s2 = material.clone();
        let mat_for_aperture = material.clone();

        const EPS: f32 = 1e-3;
        let half_thickness = center_thickness / 2.0;

        // 頂点位置
        let v1_z = -half_thickness;
        let v2_z = half_thickness;

        // 球心位置の計算（修正版）：
        // - r > 0: 凸面（球心は頂点から外向き）
        // - r < 0: 凹面（球心は頂点から内向き）
        let s1 = if r1.is_finite() {
            let center1 = Vec3::new(0.0, 0.0, v1_z + r1);
            let radius1 = r1.abs();
            eprintln!("Front surface: center_z={}, radius={}", center1.z, radius1);
            Box::new(Sphere {
                center: center1,
                radius: radius1,
                material: mat_for_s1,
            }) as Box<dyn Hittable>
        } else {
            eprintln!("Front surface: plane at z={}", v1_z);
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, v1_z),
                normal: -Vec3::Z,
                material: mat_for_s1,
            }) as Box<dyn Hittable>
        };

        let s2 = if r2.is_finite() {
            let center2 = Vec3::new(0.0, 0.0, v2_z - r2);
            let radius2 = r2.abs();
            eprintln!("Back surface: center_z={}, radius={}", center2.z, radius2);
            Box::new(Sphere {
                center: center2,
                radius: radius2,
                material: mat_for_s2,
            }) as Box<dyn Hittable>
        } else {
            eprintln!("Back surface: plane at z={}", v2_z);
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, v2_z),
                normal: Vec3::Z,
                material: mat_for_s2,
            }) as Box<dyn Hittable>
        };

        // 交差判定（デバッグ用）
        if r1.is_finite() && r2.is_finite() {
            let center1_z = v1_z + r1;
            let center2_z = v2_z - r2;
            let center_dist = (center2_z - center1_z).abs();
            let radii_sum = r1.abs() + r2.abs();
            eprintln!("Lens geometry check:");
            eprintln!("  Center distance: {}", center_dist);
            eprintln!("  Sum of radii: {}", radii_sum);
            if center_dist >= radii_sum {
                eprintln!("Warning: spheres do not intersect - lens may be empty");
            }
        }

        let infinite_lens = Box::new(CSGObject {
            left: s1,
            right: s2,
            operation: CsgOperation::Intersection,
        });

        // 開口を InfiniteCylinder に戻す（より正確な形状に）
        let axis_dir = Vec3::Z.normalize_or_zero();
        let aperture = Box::new(InfiniteCylinder {
            axis_point: Vec3::ZERO,
            axis_dir,
            radius: (diameter / 2.0) + EPS,
            material: mat_for_aperture,
        });

        let final_lens = Box::new(CSGObject {
            left: aperture,
            right: infinite_lens,
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
