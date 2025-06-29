use csv::Writer;
use glam::{Mat4, Vec3}; // 外部クレートのインポート
use rand::Rng;
use std::error::Error; // 標準ライブラリのインポート
mod primitives;
use primitives::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct SimulationSettingsConfig {
    infinity_distance: f32,
    max_bounces: u32,
}

#[derive(Deserialize)]
struct SceneConfig {
    simulation_settings: SimulationSettingsConfig,
    rays: Vec<RayConfig>,
    objects: Vec<ObjectConfig>,
}

#[derive(Deserialize)]
struct ObjectConfig {
    shape: ShapeConfig,
    material: MaterialConfig,
    transform: TransformConfig,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ShapeConfig {
    Sphere {
        radius: f32,
    },
    Box {
        size: [f32; 3],
    },
    Plane {
        normal: [f32; 3],
    },
    Cylinder {
        height: f32,
        radius: f32,
    },
    Cone {
        angle_deg: f32,
        height: f32,
    }, // 有限円錐の定義を追加
    Wedge {
        size: [f32; 3],
        angle_deg: f32,
    },
    Lens {
        thickness: f32,
        diameter: f32,
        r1: f32,
        r2: f32,
    },
}

#[derive(Deserialize, Clone, Copy)] // 材質はコピーするのでClone, Copyも
#[serde(tag = "type")]
enum MaterialConfig {
    Glass { ior: f32 },
    HalfMirror { reflectance: f32 },
    Mirror,
}

#[derive(Deserialize, Clone, Copy)]
struct GlassConfig {
    ior: f32,
}

#[derive(Deserialize)]
struct TransformConfig {
    position: [f32; 3],
    rotation_y_deg: f32,
}

#[derive(Deserialize)]
struct RayConfig {
    origin: [f32; 3],
    direction: [f32; 3],
}

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
    let toml_str = std::fs::read_to_string("scene.toml")?;
    let scene_config: SceneConfig = toml::from_str(&toml_str)?;

    let mut scene: Vec<Box<dyn Hittable>> = Vec::new();

    // --- 読み込んだ設定から動的にオブジェクトを構築 ---
    for object_config in scene_config.objects {
        let material = match object_config.material {
            MaterialConfig::Mirror {} => Material::Mirror,
            MaterialConfig::Glass { ior } => Material::Glass { ior },
            MaterialConfig::HalfMirror { reflectance } => Material::HalfMirror { reflectance },
        };

        let primitive: Box<dyn Hittable> = match object_config.shape {
            ShapeConfig::Sphere { radius } => Box::new(Sphere {
                center: Vec3::ZERO,
                radius,
                material,
            }),
            ShapeConfig::Box { size } => {
                let s = Vec3::from_array(size) / 2.0;
                Box::new(AxisAlignedBox {
                    min: -s,
                    max: s,
                    material,
                })
            }
            ShapeConfig::Plane { normal } => Box::new(Plane {
                point: Vec3::ZERO,
                normal: Vec3::from_array(normal),
                material,
            }),
            ShapeConfig::Cylinder { height, radius } => {
                let half_height = height / 2.0;
                let body = Box::new(InfiniteCylinder {
                    axis_point: Vec3::ZERO,
                    axis_dir: Vec3::Y,
                    radius,
                    material,
                });
                let cap_top = Box::new(Plane {
                    point: Vec3::new(0.0, half_height, 0.0),
                    normal: Vec3::NEG_Y,
                    material,
                });
                let cap_bottom = Box::new(Plane {
                    point: Vec3::new(0.0, -half_height, 0.0),
                    normal: Vec3::Y,
                    material,
                });
                let capped_cylinder = Box::new(CSGObject {
                    left: body,
                    right: cap_top,
                    operation: CsgOperation::Intersection,
                });
                Box::new(CSGObject {
                    left: capped_cylinder,
                    right: cap_bottom,
                    operation: CsgOperation::Intersection,
                })
            }
            ShapeConfig::Cone { angle_deg, height } => {
                let cone = Box::new(InfiniteCone::new(
                    Vec3::ZERO,
                    Vec3::Y,
                    angle_deg.to_radians(),
                    material,
                ));
                let cap = Box::new(Plane {
                    point: Vec3::new(0.0, height, 0.0),
                    normal: Vec3::NEG_Y,
                    material,
                });
                Box::new(CSGObject {
                    left: cone,
                    right: cap,
                    operation: CsgOperation::Intersection,
                })
            }
            ShapeConfig::Wedge { size, angle_deg } => Box::new(Wedge::new(
                Vec3::from_array(size),
                angle_deg.to_radians(),
                material,
            )),
            ShapeConfig::Lens {
                thickness,
                diameter,
                r1,
                r2,
            } => Box::new(Lens::new(thickness, diameter, r1, r2, material)),
        };

        // Transformを適用
        let transform_config = object_config.transform;
        let translation = Mat4::from_translation(Vec3::from_array(transform_config.position));
        let rotation = Mat4::from_rotation_y(transform_config.rotation_y_deg.to_radians());
        let transform_matrix = translation * rotation;

        scene.push(Box::new(Transform::new(primitive, transform_matrix)));
    }

    // --- 3. 初期光線の設定
    for (i, ray_config) in scene_config.rays.iter().enumerate() {
        // --- 3a. 初期光線の設定 ---
        let mut ray = Ray {
            origin: Vec3::from_array(ray_config.origin),
            direction: Vec3::from_array(ray_config.direction).normalize(),
            current_ior: 1.0,
        };

        // --- 3b. 光路の追跡 ---
        let mut path_points: Vec<Vec3> = vec![ray.origin];
        let settings = &scene_config.simulation_settings;
        let max_bounces = settings.max_bounces;
        let infinity_distance = settings.infinity_distance;

        for _ in 0..max_bounces {
            let mut closest_hit_record: Option<HitRecord> = None;
            let mut t_closest = f32::INFINITY;

            for object in &scene {
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
