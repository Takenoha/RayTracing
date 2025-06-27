use rand::Rng;
use std::error::Error;
// 3Dベクトルを扱うためにglamクレートのVec3をインポート
use glam::{Mat4, Vec3, Vec4};

// 光線を表す構造体
// origin: 始点, direction: 方向
struct Ray {
    origin: Vec3,
    direction: Vec3,
    current_ior: f32,
}

// 衝突（ヒット）に関する情報をまとめる構造体
#[derive(Debug, Clone, Copy)]
struct HitRecord {
    t: f32,
    point: Vec3,
    normal: Vec3,
    front_face: bool,
    material: Material,
}

trait Hittable {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Material {
    Mirror,
    Glass { ior: f32 },
    HalfMirror { reflectance: f32 },
}
// ============== 3D用の物理計算関数 ==============

// 反射ベクトルを計算
fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

// 屈折ベクトルを計算（全反射の可能性も考慮）
fn refract(incident: Vec3, normal: Vec3, ior_ratio: f32) -> Option<Vec3> {
    let cos_theta = (-incident).dot(normal).min(1.0);
    let sin_theta_squared = 1.0 - cos_theta * cos_theta;

    if ior_ratio * ior_ratio * sin_theta_squared > 1.0 {
        return None; // 全反射
    }

    let perp = ior_ratio * (incident + cos_theta * normal);
    let parallel = -(1.0 - perp.length_squared()).abs().sqrt() * normal;

    Some((perp + parallel).normalize())
}

// ブーリアン演算の種類
#[derive(Debug, Copy, Clone, PartialEq)]
enum CsgOperation {
    Union,        // 和集合
    Intersection, // 積集合
    Difference,   // 差集合
}

// CSGオブジェクト
struct CSGObject {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    operation: CsgOperation,
}

impl Hittable for CSGObject {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        // 1. 左右の子オブジェクトとの全ての交点を取得
        let hits_left = self
            .left
            .intersect_all(ray, t_min, t_max)
            .unwrap_or_default();
        let hits_right = self
            .right
            .intersect_all(ray, t_min, t_max)
            .unwrap_or_default();

        // 2. 全てのヒットを一つのリストにまとめ、tでソート
        let mut all_hits = hits_left.clone();
        all_hits.extend(hits_right.clone());
        all_hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

        let mut result_hits = Vec::new();

        // 3. 演算の種類に応じたフィルタリング処理
        let mut in_left = false;
        let mut in_right = false;

        for hit in &all_hits {
            // このヒットがleft/rightどちらの物か判定
            let hit_is_on_left = hits_left.iter().any(|h| (h.t - hit.t).abs() < 1e-6);

            // 演算前の状態を保存
            let was_inside = match self.operation {
                CsgOperation::Union => in_left || in_right,
                CsgOperation::Intersection => in_left && in_right,
                CsgOperation::Difference => in_left && !in_right,
            };

            // 内外状態を更新
            if hit_is_on_left {
                in_left = !in_left;
            } else {
                in_right = !in_right;
            }

            // 演算後の状態を計算
            let is_inside = match self.operation {
                CsgOperation::Union => in_left || in_right,
                CsgOperation::Intersection => in_left && in_right,
                CsgOperation::Difference => in_left && !in_right,
            };

            // 状態が変化した（＝CSGオブジェクトの表面を通過した）なら、そのヒットは有効
            if was_inside != is_inside {
                // Differenceの場合、rightオブジェクトの法線は反転させる必要がある
                if self.operation == CsgOperation::Difference && !hit_is_on_left {
                    let mut inverted_hit = *hit;
                    inverted_hit.normal = -hit.normal;
                    inverted_hit.front_face = !hit.front_face;
                    result_hits.push(inverted_hit);
                } else {
                    result_hits.push(*hit);
                }
            }
        }

        if result_hits.is_empty() {
            None
        } else {
            Some(result_hits)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    use rand::Rng;
    let mut scene: Vec<Box<dyn Hittable>> = Vec::new();

    // --- 2. 初期光線の設定 ---
    let mut ray = Ray {
        origin: Vec3::new(-20.0, 2.0, 0.0),
        direction: Vec3::new(1.0, 0.0, 0.0).normalize(),
        current_ior: 1.0, // 空気からスタート
    };

    // --- 3. 光路の追跡 ---
    let mut path_points: Vec<Vec3> = vec![ray.origin];
    let max_bounces = 10;

    for _ in 0..max_bounces {
        let mut closest_hit_record: Option<HitRecord> = None;
        let mut t_closest = f32::INFINITY;

        for object in &scene {
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

                    if let Some(refracted_dir) = refract(ray.direction, hit.normal, ior_ratio) {
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
            path_points.push(ray.origin + ray.direction * 200.0);
            break;
        }
    }

    // --- 4. 結果をCSVファイルに出力 ---
    let mut wtr = csv::Writer::from_path("path_3d.csv")?;
    wtr.write_record(&["x", "y", "z"])?; // ヘッダー
    for point in path_points {
        wtr.write_record(&[
            point.x.to_string(),
            point.y.to_string(),
            point.z.to_string(),
        ])?;
    }
    wtr.flush()?;

    println!("3D光路を 'path_3d.csv' に出力しました。");
    println!("可視化スクリプトで結果を確認してください。");

    Ok(())
}
