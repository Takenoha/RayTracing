use crate::{HitRecord, Hittable, Material, Ray, RenderableShape};
use glam::Vec3;
// 無限円錐
#[derive(Debug, Clone, Copy)]
pub struct InfiniteCone {
    pub vertex: Vec3,      // 円錐の頂点
    pub axis_dir: Vec3,    // 軸の方向（正規化されていること）
    pub cos_angle_sq: f32, // 開き角度のコサインの2乗 (cos^2(α))
    pub material: Material,
}

impl InfiniteCone {
    // 角度(ラジアン)からcos^2(α)を計算するコンストラクタ
    pub fn new(vertex: Vec3, axis_dir: Vec3, angle_rad: f32, material: Material) -> Self {
        Self {
            vertex,
            axis_dir: axis_dir.normalize(),
            cos_angle_sq: angle_rad.cos().powi(2),
            material,
        }
    }
}

// InfiniteCone のための Hittable 実装
impl Hittable for InfiniteCone {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let co = ray.origin - self.vertex; // 頂点からレイの始点へのベクトル

        let d_dot_v = ray.direction.dot(self.axis_dir);
        let co_dot_v = co.dot(self.axis_dir);

        // 二次方程式の係数 A, B, C を計算
        // A = (D・V)^2 - cos^2(α)
        // B = 2 * [ (D・V)(CO・V) - (D・CO)cos^2(α) ]
        // C = (CO・V)^2 - (CO・CO)cos^2(α)
        // (Dは正規化済みなので D・D = 1 と仮定)
        let a = d_dot_v.powi(2) - self.cos_angle_sq;
        let b = 2.0 * (d_dot_v * co_dot_v - ray.direction.dot(co) * self.cos_angle_sq);
        let c = co_dot_v.powi(2) - co.length_squared() * self.cos_angle_sq;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None; // 実数解なし
        }

        let sqrtd = discriminant.sqrt();
        let mut hits = Vec::new();

        // 2つの解を計算
        let t1 = (-b - sqrtd) / (2.0 * a);
        let t2 = (-b + sqrtd) / (2.0 * a);

        for &t in &[t1, t2] {
            if t > t_min && t < t_max {
                let point = ray.origin + t * ray.direction;

                // 法線を計算
                // N = normalize( (P-V) - (1+tan^2(α)) * ((P-V)・V) * V ) を元に計算
                // より単純な勾配法 N = normalize( (PV・v)v - cos²(α)PV ) を使う
                let pv = point - self.vertex;
                let m = pv.dot(self.axis_dir);
                let outward_normal = (m * self.axis_dir - pv * self.cos_angle_sq).normalize();

                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };

                hits.push(HitRecord {
                    t,
                    point,
                    normal,
                    front_face,
                    material: self.material,
                });
            }
        }

        if hits.is_empty() {
            None
        } else {
            Some(hits)
        }
    }

    fn get_renderable_shape(&self) -> Option<RenderableShape> {
        // This is an infinite shape, so we don't render it directly.
        // It will be rendered as part of a CSG object.
        None
    }

    fn get_transform(&self) -> glam::Mat4 {
        glam::Mat4::IDENTITY
    }
}
