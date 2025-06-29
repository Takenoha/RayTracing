use crate::{HitRecord, Hittable, Material, Ray};
use glam::Mat4;
// 他のHittableオブジェクトに変換を適用するためのラッパー
pub struct Transform {
    pub object: Box<dyn Hittable>,
    pub transform: Mat4,         // ローカル空間 -> ワールド空間への変換
    pub inverse_transform: Mat4, // ワールド空間 -> ローカル空間への変換
}

impl Transform {
    pub fn new(object: Box<dyn Hittable>, transform: Mat4) -> Self {
        Self {
            object,
            transform,
            inverse_transform: transform.inverse(), // 逆行列も保持
        }
    }
}
impl Hittable for Transform {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        // 1. レイをワールド空間からオブジェクトのローカル空間へ逆変換
        let local_ray_origin = self.inverse_transform.transform_point3(ray.origin);
        let local_ray_direction = self.inverse_transform.transform_vector3(ray.direction);
        let local_ray = Ray {
            origin: local_ray_origin,
            direction: local_ray_direction,
            current_ior: ray.current_ior, // IORは空間変換で変化しない
        };

        // 2. ローカル空間で、包み込んだオブジェクトとの交差判定を行う
        if let Some(local_hits) = self.object.intersect_all(&local_ray, t_min, t_max) {
            // 3. 結果をローカル空間からワールド空間へ変換して返す
            let world_hits = local_hits
                .into_iter()
                .map(|mut hit| {
                    // 衝突点と法線をワールド空間に変換
                    hit.point = self.transform.transform_point3(hit.point);
                    // 法線ベクトルの変換は、逆行列の転置行列をかけるのが数学的に正しい
                    hit.normal = self
                        .inverse_transform
                        .transpose()
                        .transform_vector3(hit.normal)
                        .normalize();
                    hit
                })
                .collect();
            Some(world_hits)
        } else {
            None
        }
    }
}
