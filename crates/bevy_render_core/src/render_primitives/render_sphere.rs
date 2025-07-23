use crate::{HitRecord, Hittable, Material, Ray};
use glam::Vec3; // main.rsから移動させる共通定義をインポート

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    // ★ pub を追加
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}
impl Hittable for Sphere {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut hits = Vec::new();

        // 1つ目の解
        let t1 = (-half_b - sqrtd) / a;
        if t1 > t_min && t1 < t_max {
            let point = ray.origin + t1 * ray.direction;
            let outward_normal = (point - self.center) / self.radius;
            let front_face = ray.direction.dot(outward_normal) < 0.0;
            let normal = if front_face {
                outward_normal
            } else {
                -outward_normal
            };

            hits.push(HitRecord {
                t: t1,
                point,
                normal,
                front_face,
                material: self.material,
            });
        }

        // 2つ目の解
        if discriminant > 1e-6 {
            let t2 = (-half_b + sqrtd) / a;
            if t2 > t_min && t2 < t_max {
                let point = ray.origin + t2 * ray.direction;
                let outward_normal = (point - self.center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };

                hits.push(HitRecord {
                    t: t2,
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
}
