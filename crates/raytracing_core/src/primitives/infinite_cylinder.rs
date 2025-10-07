use crate::{AABB, HitRecord, Hittable, Material, Ray};
use glam::Vec3;

// 無限円柱
#[derive(Debug, Clone, Copy)]
pub struct InfiniteCylinder {
    pub axis_point: Vec3, // 軸上の任意の点
    pub axis_dir: Vec3,   // 軸の方向（正規化されていること）
    pub radius: f32,
    pub material: Material,
}

impl Hittable for InfiniteCylinder {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let eps = 1e-8;

        // 安全化：軸ベクトルは正規化（零ベクトルなら早期終了）
        let axis_dir = self.axis_dir.normalize_or_zero();
        if axis_dir.length_squared() < eps {
            eprintln!("InfiniteCylinder: axis_dir nearly zero -> skipping intersect");
            return None;
        }

        let oc = ray.origin - self.axis_point;

        let d_dot_v = ray.direction.dot(axis_dir);
        let d_perp = ray.direction - d_dot_v * axis_dir;

        let oc_dot_v = oc.dot(axis_dir);
        let oc_perp = oc - oc_dot_v * axis_dir;

        let a = d_perp.length_squared();
        let b = 2.0 * oc_perp.dot(d_perp);
        let c = oc_perp.length_squared() - self.radius * self.radius;

        // 軸に平行に近い場合は別処理（現状は交差なし）
        if a.abs() < eps {
            return None;
        }

        let discriminant = b * b - 4.0 * a * c;
        if !discriminant.is_finite() || discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut hits = Vec::new();
        let candidates = [(-b - sqrtd) / (2.0 * a), (-b + sqrtd) / (2.0 * a)];

        for &t in &candidates {
            if !t.is_finite() {
                continue;
            }
            if t <= t_min || t >= t_max {
                continue;
            }

            let point = ray.origin + t * ray.direction;

            // 法線：点から軸へのベクトル
            let v = point - self.axis_point;
            let proj = v.dot(axis_dir);
            let point_on_axis = self.axis_point + proj * axis_dir;
            let mut outward = point - point_on_axis;
            let len2 = outward.length_squared();
            if len2 < eps {
                // 軸上に近いなら安定な任意法線を作る
                outward = if axis_dir.cross(Vec3::X).length_squared() > eps {
                    axis_dir.cross(Vec3::X).normalize_or_zero()
                } else {
                    axis_dir.cross(Vec3::Y).normalize_or_zero()
                };
            } else {
                outward = outward / len2.sqrt();
            }

            let front_face = ray.direction.dot(outward) < 0.0;
            let normal = if front_face { outward } else { -outward };

            hits.push(HitRecord {
                t,
                point,
                normal,
                front_face,
                material: self.material, // .clone() が必要なら変更
            });
        }

        if hits.is_empty() { None } else { Some(hits) }
    }

    fn bounding_box(&self) -> Option<AABB> {
        None
    }

    fn clone_hittable(&self) -> Box<dyn Hittable> {
        Box::new(*self)
    }
}
