use csv::Writer;
use glam::{Mat4, Vec3}; // 外部クレートのインポート
use rand::Rng;
use std::error::Error; // 標準ライブラリのインポート
mod primitives;
use primitives::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct SceneConfig {
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
#[serde(tag = "type")] // TOMLのtype="Lens"などでどのenumかを判断
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

#[derive(Deserialize, Clone)] // 材質はコピーするのでClone, Copyも
#[serde(untagged)] // "Mirror"のような単純な文字列か、{...}テーブルかを自動判断
enum MaterialConfig {
    Mirror {}, // パラメータがない場合は、バリアント名だけでOK
    Glass { ior: f32 },
    HalfMirror { reflectance: f32 },
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
