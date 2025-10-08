use crate::{AABB, HitRecord, Hittable, Material, Ray};
use glam::Vec3;
// 軸並行な直方体 (AABB) 対角の座標を指定
#[derive(Debug, Clone, Copy)]
pub struct AxisAlignedBox {
    pub min: Vec3, // 3つの軸の最小座標 (x_min, y_min, z_min)
    pub max: Vec3, // 3つの軸の最大座標 (x_max, y_max, z_max)
    pub material: Material,
}
// AxisAlignedBox のための Hittable 実装
impl Hittable for AxisAlignedBox {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let eps = 1e-8;
        let mut t0 = t_min;
        let mut t1 = t_max;

        // 各軸でスラブを計算。dir が 0 の場合は原点成分が範囲内か確認する
        for i in 0..3 {
            let origin_comp = ray.origin[i];
            let dir_comp = ray.direction[i];
            let min_comp = self.min[i];
            let max_comp = self.max[i];

            if dir_comp.abs() < eps {
                // 平行 -> origin が slab の間にあるかを確認
                if origin_comp < min_comp - eps || origin_comp > max_comp + eps {
                    return None;
                } else {
                    continue;
                }
            }

            let inv = 1.0 / dir_comp;
            let mut t_near = (min_comp - origin_comp) * inv;
            let mut t_far = (max_comp - origin_comp) * inv;
            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
            }

            t0 = t0.max(t_near);
            t1 = t1.min(t_far);
            if t1 <= t0 {
                return None;
            }
        }

        // t0 が第一交点、t1 が第二交点
        let mut hits = Vec::new();
        for &t in &[t0, t1] {
            if !t.is_finite() {
                continue;
            }
            let point = ray.origin + t * ray.direction;
            // 法線推定：どの軸で境界に近いかで決める（単純化）
            let mut normal = Vec3::ZERO;
            for axis in 0..3 {
                if (point[axis] - self.min[axis]).abs() < 1e-4 {
                    let mut n = Vec3::ZERO;
                    n[axis] = -1.0;
                    normal = n;
                    break;
                }
                if (point[axis] - self.max[axis]).abs() < 1e-4 {
                    let mut n = Vec3::ZERO;
                    n[axis] = 1.0;
                    normal = n;
                    break;
                }
            }
            if normal == Vec3::ZERO {
                // 安定化 fallback
                normal = (point - (self.min + self.max) * 0.5).normalize_or_zero();
            }
            let front_face = ray.direction.dot(normal) < 0.0;
            let normal = if front_face { normal } else { -normal };
            hits.push(HitRecord {
                t,
                point,
                normal,
                front_face,
                material: self.material,
            });
        }

        if hits.is_empty() { None } else { Some(hits) }
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB {
            min: self.min,
            max: self.max,
        })
    }

    fn clone_hittable(&self) -> Box<dyn Hittable> {
        Box::new(*self)
    }
}

// AABBのためのヘルパーメソッド
impl AxisAlignedBox {
    // 衝突点から、どの面の法線かを計算する
    #[allow(dead_code)]
    fn calculate_normal(&self, point: Vec3) -> Vec3 {
        let epsilon = 1e-4;
        let p_minus_min = point - self.min;
        let p_minus_max = point - self.max;

        if p_minus_min.x.abs() < epsilon {
            return Vec3::NEG_X;
        }
        if p_minus_max.x.abs() < epsilon {
            return Vec3::X;
        }
        if p_minus_min.y.abs() < epsilon {
            return Vec3::NEG_Y;
        }
        if p_minus_max.y.abs() < epsilon {
            return Vec3::Y;
        }
        if p_minus_min.z.abs() < epsilon {
            return Vec3::NEG_Z;
        }
        if p_minus_max.z.abs() < epsilon {
            return Vec3::Z;
        }

        Vec3::ZERO // 本来は到達しない
    }
}
