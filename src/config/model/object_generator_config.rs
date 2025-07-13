use glam::Vec3;
use serde::Deserialize;

use crate::model::MaterialConfig;
use crate::model::ObjectConfig;
use crate::model::SceneConfig;
use crate::model::ShapeConfig;
use crate::model::SimulationConfig;
use crate::model::SimulationSettingsConfig;
use crate::Hittable;
use crate::Ray;
use crate::Scene;
use crate::Transform;

// --- ジェネレータの定義 ---

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RayGeneratorConfig {
    ParallelGrid {
        origin_corner: [f32; 3],
        vec_u: [f32; 3],
        vec_v: [f32; 3],
        count_u: u32,
        count_v: u32,
        direction: [f32; 3],
        current_ior: f32,
    },
    Projector {
        origin: [f32; 3],
        target_corner: [f32; 3],
        target_u: [f32; 3],
        target_v: [f32; 3],
        count_u: u32,
        count_v: u32,
        current_ior: f32,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ObjectGeneratorConfig {
    ObjectGrid {
        count_x: u32,
        count_z: u32,
        position_start: [f32; 3],
        step_x: [f32; 3],
        step_z: [f32; 3],
        template: ObjectConfig, // オブジェクトのテンプレート
    },
}

#[derive(Deserialize, Clone)] // テンプレートはクローン可能にする
pub struct ObjectTemplateConfig {
    pub shape: ShapeConfig,
    pub material: MaterialConfig,
}

// --- シーン全体のコンフィグ ---
// (ShapeConfig, MaterialConfig, ObjectConfig などは以前のものを使用)

#[derive(Deserialize)]
pub struct SceneDefinition {
    // defaultを追加して、TOMLにキーが無くてもエラーにならないようにする
    //#[serde(default)]
    pub ray_generators: Vec<RayGeneratorConfig>,

    //#[serde(default)]
    pub object_generators: Vec<ObjectGeneratorConfig>,

    //#[serde(default)]
    pub objects: Vec<ObjectConfig>,
}

// この関数でConfigから実行時に使うオブジェクトを生成する
pub fn build_scene_from_config(config: SceneDefinition) -> (Vec<Ray>, Vec<Box<dyn Hittable>>) {
    let mut rays: Vec<Ray> = Vec::new();
    let mut hittables: Vec<Box<dyn Hittable>> = Vec::new();

    // === レイの生成 ===
    for generator in config.ray_generators {
        match generator {
            RayGeneratorConfig::ParallelGrid {
                origin_corner,
                vec_u,
                vec_v,
                count_u,
                count_v,
                direction,
                current_ior,
            } => {
                let corner = Vec3::from(origin_corner);
                let u_step = Vec3::from(vec_u) / (count_u as f32);
                let v_step = Vec3::from(vec_v) / (count_v as f32);
                let dir = Vec3::from(direction).normalize();

                for i in 0..count_u {
                    for j in 0..count_v {
                        let origin = corner + (i as f32 * u_step) + (j as f32 * v_step);
                        rays.push(Ray {
                            origin,
                            direction: dir,
                            current_ior, //..Default::default()
                        });
                    }
                }
            }
            RayGeneratorConfig::Projector {
                origin,
                target_corner,
                target_u,
                target_v,
                count_u,
                count_v,
                current_ior,
            } => {
                let ray_origin = Vec3::from(origin);
                let target_c = Vec3::from(target_corner);
                let target_u_step = Vec3::from(target_u) / (count_u as f32);
                let target_v_step = Vec3::from(target_v) / (count_v as f32);

                for i in 0..count_u {
                    for j in 0..count_v {
                        let target_point =
                            target_c + (i as f32 * target_u_step) + (j as f32 * target_v_step);
                        rays.push(Ray {
                            origin: ray_origin,
                            direction: (target_point - ray_origin).normalize(),
                            current_ior, //..Default::default()
                        });
                    }
                }
            }
        }
    }

    // === オブジェクトの生成 ===
    for generator in config.object_generators {
        match generator {
            ObjectGeneratorConfig::ObjectGrid {
                count_x,
                count_z,
                position_start,
                step_x,
                step_z,
                template,
            } => {
                let start_pos = Vec3::from(position_start);
                let x_step = Vec3::from(step_x);
                let z_step = Vec3::from(step_z);

                for i in 0..count_x {
                    for j in 0..count_z {
                        let pos = start_pos + (i as f32 * x_step) + (j as f32 * z_step);

                        // テンプレートを複製し、transform.positionのみ変更
                        let mut obj = template.clone();
                        obj.transform.position = pos.to_array();

                        let hittable: Box<dyn Hittable> = obj.into();
                        hittables.push(hittable);
                    }
                }
            }
        }
    }

    // === 個別オブジェクトの追加 ===
    for obj_conf in config.objects {
        let material = obj_conf.material.into();
        let mut hittable = obj_conf.shape.into_with(material);
        // ... transformの適用 ...
        hittables.push(hittable);
    }

    (rays, hittables)
}

// 以下は仮のヘルパー関数
// pub fn build_material(config: MaterialConfig) -> Material { ... }
// impl ShapeConfig { pub fn into_with(self, material: Material) -> Box<dyn Hittable> { ... } }
