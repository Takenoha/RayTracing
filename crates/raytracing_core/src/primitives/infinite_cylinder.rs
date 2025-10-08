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
    fn intersect_all(&self, ray: &Ray, _t_min: f32, _t_max: f32) -> Option<Vec<HitRecord>> {
        let eps = 1e-8;

        // デバッグ出力を追加
        //eprintln!("InfiniteCylinder: processing ray {:?}", ray);

        // 軸の正規化と検証
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

        // 2次方程式の係数
        let a = d_perp.length_squared();
        let b = 2.0 * oc_perp.dot(d_perp);
        let c = oc_perp.length_squared() - self.radius * self.radius;

        // 非有限値チェックを追加
        if !a.is_finite() || !b.is_finite() || !c.is_finite() {
            eprintln!(
                "InfiniteCylinder: non-finite coefficients: a={}, b={}, c={}",
                a, b, c
            );
            return None;
        }

        // 軸平行の場合の特別処理
        if a.abs() < eps {
            let dist_to_axis = oc_perp.length();
            if dist_to_axis <= self.radius {
                // 軸内部からの射出は交点なし
                eprintln!("InfiniteCylinder: ray parallel to axis, inside cylinder");
                return None;
            } else {
                // 軸外部からの射出も交点なし
                eprintln!("InfiniteCylinder: ray parallel to axis, outside cylinder");
                return None;
            }
        }

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut hits = Vec::new();
        let candidates = [(-b - sqrtd) / (2.0 * a), (-b + sqrtd) / (2.0 * a)];

        // すべての交点を収集（CSGと同様）
        for &t in &candidates {
            if !t.is_finite() {
                eprintln!("InfiniteCylinder: dropped non-finite t = {:?}", t);
                continue;
            }

            // t_min/t_maxの範囲チェックを緩和（CSGのため）
            let point = ray.origin + t * ray.direction;

            // 法線計算を改善
            let v = point - self.axis_point;
            let proj = v.dot(axis_dir);
            let point_on_axis = self.axis_point + proj * axis_dir;
            let outward_dir = point - point_on_axis;

            let normal = if outward_dir.length_squared() < eps {
                // 軸上での安定な法線生成
                let temp = if axis_dir.cross(ray.direction).length_squared() > eps {
                    axis_dir.cross(ray.direction)
                } else {
                    axis_dir.cross(Vec3::X)
                };
                temp.cross(axis_dir).normalize_or_zero()
            } else {
                outward_dir.normalize_or_zero()
            };

            let front_face = ray.direction.dot(normal) < 0.0;

            hits.push(HitRecord {
                t,
                point,
                normal: if front_face { normal } else { -normal },
                front_face,
                material: self.material,
            });
        }

        // ヒットを t でソート
        if !hits.is_empty() {
            hits.sort_by(|a, b| a.t.total_cmp(&b.t));
            //eprintln!("InfiniteCylinder: found {} hits", hits.len());
            Some(hits)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Option<AABB> {
        None
    }

    fn clone_hittable(&self) -> Box<dyn Hittable> {
        Box::new(*self)
    }
}
