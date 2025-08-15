use crate::{HitRecord, Hittable, Material, Ray, RenderableShape};
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
        let mut tmin = t_min;
        let mut tmax = t_max;

        // 各軸 (X, Y, Z) に対してSlab Testを実行
        for i in 0..3 {
            // レイの方向の逆数。ゼロ除算を避ける
            let inv_d = 1.0 / ray.direction[i];
            let mut t0 = (self.min[i] - ray.origin[i]) * inv_d;
            let mut t1 = (self.max[i] - ray.origin[i]) * inv_d;

            // レイの進行方向に応じて、t0とt1（スラブへの入口と出口）を入れ替える
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            // これまで計算された全体の区間と、現在の軸の区間の共通部分を求める
            tmin = tmin.max(t0);
            tmax = tmax.min(t1);

            // 共通区間がなくなれば、ヒットしない
            if tmax <= tmin {
                return None;
            }
        }

        // --- 有効な交差区間 [tmin, tmax] が見つかった ---
        let mut hits = Vec::new();

        // 最初のヒット (入口)
        let point1 = ray.origin + tmin * ray.direction;
        let normal1 = self.calculate_normal(point1);
        hits.push(HitRecord {
            t: tmin,
            point: point1,
            normal: normal1,
            front_face: ray.direction.dot(normal1) < 0.0,
            material: self.material,
        });

        // 2番目のヒット (出口)
        let point2 = ray.origin + tmax * ray.direction;
        let normal2 = -self.calculate_normal(point2); // 出口の法線は内側を向く
        hits.push(HitRecord {
            t: tmax,
            point: point2,
            normal: normal2,
            front_face: ray.direction.dot(normal2) < 0.0,
            material: self.material,
        });

        Some(hits)
    }

    fn get_renderable_shape(&self) -> Option<RenderableShape> {
        Some(RenderableShape::Box {
            size: self.max - self.min,
        })
    }

    fn get_transform(&self) -> glam::Mat4 {
        glam::Mat4::IDENTITY
    }
}

// AABBのためのヘルパーメソッド
impl AxisAlignedBox {
    // 衝突点から、どの面の法線かを計算する
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
