use std::error::Error; // 標準ライブラリのインポート

use config::model::SceneConfig;
use csv::Writer;
use glam::Vec3; // 外部クレートのインポート
use primitives::*;
use rand::Rng;

mod config;
mod primitives;

// 光線を表す構造体
// origin: 始点, direction: 方向
#[derive(Debug)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    current_ior: f32,
}

// 衝突（ヒット）に関する情報をまとめる構造体
#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    t: f32,
    point: Vec3,
    normal: Vec3,
    front_face: bool,
    material: Material,
}

pub trait Hittable {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
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
        println!("全反射が発生しました！ ior_ratio: {}", ior_ratio);
        return None; // 全反射
    }

    let perp = ior_ratio * (incident + cos_theta * normal);
    let parallel = -(1.0 - perp.length_squared()).abs().sqrt() * normal;

    Some((perp + parallel).normalize())
}

// ブーリアン演算の種類
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CsgOperation {
    Union,        // 和集合
    Intersection, // 積集合
    Difference,   // 差集合
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("設定ファイル scene.toml を読み込んでいます...");
    let scene_config = SceneConfig::load_from_path("scene.toml")?;

    // --- 読み込んだ設定から動的にオブジェクトを構築 ---
    let objects: Vec<Box<dyn Hittable>> =
        scene_config.objects.into_iter().map(Into::into).collect();

    // --- 3. 初期光線の設定
    for (i, mut ray) in scene_config
        .rays
        .into_iter()
        .map(Into::<Ray>::into)
        .enumerate()
    {
        // --- 3b. 光路の追跡 ---
        let mut path_points: Vec<Vec3> = vec![ray.origin];
        let settings = &scene_config.simulation_settings;
        let max_bounces = settings.max_bounces;
        let infinity_distance = settings.infinity_distance;

        for _ in 0..max_bounces {
            let mut closest_hit_record: Option<HitRecord> = None;
            let mut t_closest = f32::INFINITY;

            for object in &objects {
                if let Some(hits) = object.intersect_all(&ray, 0.001, t_closest) {
                    print!("{:?}", hits);
                    if let Some(first_hit) = hits.first() {
                        if first_hit.t < t_closest {
                            t_closest = first_hit.t;
                            closest_hit_record = Some(*first_hit);
                        }
                    }
                }
            }
            //print!("{:?}", closest_hit_record);
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
                path_points.push(ray.origin + ray.direction * infinity_distance);
                break;
            }
        }
        // --- 3c. 結果を光路ごとに別々のCSVファイルに出力 ---
        let file_name = format!("path_{}.csv", i);
        let mut wtr = Writer::from_path(&file_name)?;
        wtr.write_record(&["x", "y", "z"])?;
        for point in path_points {
            wtr.write_record(&[
                point.x.to_string(),
                point.y.to_string(),
                point.z.to_string(),
            ])?;
        }
        wtr.flush()?;
        println!("光路 {} を '{}' に出力しました。", i, file_name);
    }
    Ok(())
}

/*
// main関数を一時的に以下に置き換えてテストする
fn main() -> Result<(), Box<dyn Error>> {
    // --- 1. シーンのセットアップ ---
    // テストしやすいように、原点にあるガラス球だけを置く
    let mut scene: Vec<Box<dyn Hittable>> = Vec::new();
    let glass_sphere = Box::new(Sphere {
        center: Vec3::ZERO,
        radius: 5.0,
        material: Material::Glass { ior: 1.5 },
    });
    scene.push(glass_sphere);

    // --- 2. 初期光線の設定 ---
    let mut ray = Ray {
        origin: Vec3::new(-10.0, 2.0, 0.0),
        direction: Vec3::X, // X軸の正方向
        current_ior: 1.0,   // 空気からスタート
    };

    // --- 3. 光路の追跡（2回限定）---
    let mut path_points: Vec<Vec3> = vec![ray.origin];
    println!("追跡開始: {:?}", ray);

    // ==================== 1回目の衝突計算 ====================
    println!("\n--- 1回目の衝突計算 ---");
    let mut closest_hit_record: Option<HitRecord> = None;
    let mut t_closest = f32::INFINITY;

    for object in &scene {
        if let Some(hits) = object.intersect_all(&ray, 0.001, t_closest) {
            if let Some(first_hit) = hits.first() {
                t_closest = first_hit.t;
                closest_hit_record = Some(*first_hit);
            }
        }
    }

    if let Some(hit) = closest_hit_record {
        println!("衝突成功！（入口）");
        println!("  - 衝突点: {:?}", hit.point);
        println!("  - 法線: {:?}", hit.normal);
        println!("  - front_face: {}", hit.front_face);

        path_points.push(hit.point);
        let material = hit.material;

        if let Material::Glass { ior: material_ior } = material {
            let n1 = ray.current_ior;
            let n2 = if hit.front_face { material_ior } else { 1.0 };
            let ior_ratio = n1 / n2;

            println!("  - 屈折率: {} -> {}", n1, n2);

            if let Some(refracted_dir) = refract(ray.direction, hit.normal, ior_ratio) {
                println!("  - 屈折成功！");
                ray.direction = refracted_dir;
                ray.current_ior = n2; // 媒質情報を更新
            } else {
                println!("  - 全反射発生！");
                ray.direction = reflect(ray.direction, hit.normal);
            }
        }
        ray.origin = hit.point + ray.direction * 0.001; // 次の始点を設定
        println!("新しいレイ（内部）: {:?}", ray);
    } else {
        println!("1回目の衝突が見つかりませんでした。追跡を終了します。");
        return Ok(());
    }

    // ==================== 2回目の衝突計算 ====================
    println!("\n--- 2回目の衝突計算 ---");
    closest_hit_record = None;
    t_closest = f32::INFINITY;

    for object in &scene {
        if let Some(hits) = object.intersect_all(&ray, 0.001, t_closest) {
            if let Some(first_hit) = hits.first() {
                t_closest = first_hit.t;
                closest_hit_record = Some(*first_hit);
            }
        }
    }

    if let Some(hit) = closest_hit_record {
        println!("衝突成功！（出口）");
        println!("  - 衝突点: {:?}", hit.point);
        println!("  - 法線: {:?}", hit.normal);
        println!("  - front_face: {}", hit.front_face);

        path_points.push(hit.point);
        let material = hit.material;

        if let Material::Glass { ior: material_ior } = material {
            let n1 = ray.current_ior;
            let n2 = if hit.front_face { material_ior } else { 1.0 };
            let ior_ratio = n1 / n2;

            println!("  - 屈折率: {} -> {}", n1, n2);

            if let Some(refracted_dir) = refract(ray.direction, hit.normal, ior_ratio) {
                println!("  - 屈折成功！");
                ray.direction = refracted_dir;
                ray.current_ior = n2;
            } else {
                println!("  - 全反射発生！");
                ray.direction = reflect(ray.direction, hit.normal);
            }
        }
        ray.origin = hit.point + ray.direction * 0.001;
        path_points.push(ray.origin + ray.direction * 20.0); // 最後の光路を長く描画
        println!("最終的なレイ（外部）: {:?}", ray);
    } else {
        println!("2回目の衝突が見つかりませんでした。");
        path_points.push(ray.origin + ray.direction * 20.0);
    }

    // --- 4. 結果をコンソールに出力 ---
    println!("\n最終的な光路座標:");
    for (i, point) in path_points.iter().enumerate() {
        println!("  {}: ({:.3}, {:.3}, {:.3})", i, point.x, point.y, point.z);
    }

    Ok(())
}
*/
