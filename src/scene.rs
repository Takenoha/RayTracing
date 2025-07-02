use std::clone;

use glam::Vec3;
use rand::Rng;

use crate::{model::SimulationSettingsConfig, Hittable, Material};

// 反射ベクトルを計算
fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

// 屈折ベクトルを計算（全反射の可能性も考慮）
fn refract(incident: Vec3, normal: Vec3, ior_ratio: f32) -> Option<Vec3> {
    let cos_theta = (-incident).dot(normal).min(1.0);
    let sin_theta_squared = 1.0 - cos_theta * cos_theta;

    if ior_ratio * ior_ratio * sin_theta_squared > 1.0 {
        println!("全反射が発生しました！ ior_ratio: {}", ior_ratio);
        return None; // 全反射
    }

    let perp = ior_ratio * (incident + cos_theta * normal);
    let parallel = -(1.0 - perp.length_squared()).abs().sqrt() * normal;

    Some((perp + parallel).normalize())
}

pub struct Scene {
    pub objects: Vec<Box<dyn Hittable>>,
    pub rays: Vec<Ray>,
}

impl Scene {
    pub fn simulate_rays(&self, setting: SimulationSettingsConfig) -> Vec<Vec<Vec3>> {
        let mut result: Vec<Vec<Vec3>> = Vec::new();
        // --- 3. 初期光線の設定
        for ray in &self.rays {
            let mut ray = ray.clone();
            // --- 3b. 光路の追跡 ---
            let mut path_points: Vec<Vec3> = vec![ray.origin];
            let max_bounces = setting.max_bounces;
            let infinity_distance = setting.infinity_distance;

            for _ in 0..max_bounces {
                let mut closest_hit_record: Option<HitRecord> = None;
                let mut t_closest = f32::INFINITY;

                for object in &self.objects {
                    if let Some(hits) = object.intersect_all(&ray, 0.001, t_closest) {
                        if let Some(first_hit) = hits.first() {
                            if first_hit.t < t_closest {
                                t_closest = first_hit.t;
                                closest_hit_record = Some(*first_hit);
                            }
                        }
                    }
                }
                if let Some(hit) = closest_hit_record {
                    path_points.push(hit.point);

                    let material = hit.material; // HitRecordから直接マテリアルを取得！

                    match material {
                        Material::Mirror => {
                            ray.direction = reflect(ray.direction, hit.normal);
                        }
                        Material::Glass { ior: material_ior } => {
                            let n1 = ray.current_ior;
                            let n2 = if hit.front_face { material_ior } else { 1.0 };
                            let ior_ratio = n1 / n2;

                            if let Some(refracted_dir) =
                                refract(ray.direction, hit.normal, ior_ratio)
                            {
                                ray.direction = refracted_dir;
                                ray.current_ior = n2;
                            } else {
                                ray.direction = reflect(ray.direction, hit.normal);
                            }
                        }
                        Material::HalfMirror { reflectance } => {
                            // 0.0から1.0までの一様な乱数を生成
                            if rand::thread_rng().gen::<f32>() < reflectance {
                                // 反射する場合
                                ray.direction = reflect(ray.direction, hit.normal);
                            } else {
                                // 透過する場合（方向は変わらない）
                                // ray.direction はそのまま
                            }
                        }
                    }
                    ray.origin = hit.point + ray.direction * 0.001;
                } else {
                    path_points.push(ray.origin + ray.direction * infinity_distance);
                    break;
                }
            }
            result.push(path_points);
        }
        result
    }
}
// 光線を表す構造体
// origin: 始点, direction: 方向
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub current_ior: f32,
}

// 衝突（ヒット）に関する情報をまとめる構造体
#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub front_face: bool,
    pub material: Material,
}
