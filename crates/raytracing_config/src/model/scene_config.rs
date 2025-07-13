use std::{error::Error, path::Path};

use raytracing_core::{Hittable, Ray, Scene};
use serde::Deserialize;

use crate::{
    model::object_generator_config::{ObjectGeneratorConfig, RayGeneratorConfig},
    object_config::ObjectConfig,
    ray_config::RayConfig,
};

#[derive(Deserialize)]
pub struct SceneConfig {
    #[serde(default)]
    pub rays: Vec<RayConfig>,
    #[serde(default)]
    pub ray_generators: Vec<RayGeneratorConfig>,
    #[serde(default)]
    pub object_generators: Vec<ObjectGeneratorConfig>,
    #[serde(default)]
    pub objects: Vec<ObjectConfig>,
}

impl Into<Scene> for SceneConfig {
    fn into(self) -> Scene {
        // 個別オブジェクト
        let mut objects: Vec<Box<dyn Hittable>> =
            self.objects.into_iter().map(Into::into).collect();

        // ジェネレータから生成
        for generator in self.object_generators {
            match generator {
                ObjectGeneratorConfig::ObjectGrid {
                    count_x,
                    count_z,
                    position_start,
                    step_x,
                    step_z,
                    template,
                } => {
                    let start_pos = glam::Vec3::from(position_start);
                    let x_step = glam::Vec3::from(step_x);
                    let z_step = glam::Vec3::from(step_z);
                    for i in 0..count_x {
                        for j in 0..count_z {
                            let pos = start_pos + (i as f32 * x_step) + (j as f32 * z_step);
                            let mut obj = template.clone();
                            obj.transform.position = pos.to_array();
                            objects.push(obj.into());
                        }
                    }
                }
            }
        }

        // 個別レイ
        let mut rays: Vec<Ray> = self.rays.into_iter().map(Into::into).collect();

        // ray_generatorsから生成
        for generator in self.ray_generators {
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
                    let corner = glam::Vec3::from(origin_corner);
                    let u_step = glam::Vec3::from(vec_u) / (count_u as f32);
                    let v_step = glam::Vec3::from(vec_v) / (count_v as f32);
                    let dir = glam::Vec3::from(direction).normalize();
                    for i in 0..count_u {
                        for j in 0..count_v {
                            let origin = corner + (i as f32 * u_step) + (j as f32 * v_step);
                            rays.push(Ray {
                                origin,
                                direction: dir,
                                current_ior,
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
                    let ray_origin = glam::Vec3::from(origin);
                    let target_c = glam::Vec3::from(target_corner);
                    let target_u_step = glam::Vec3::from(target_u) / (count_u as f32);
                    let target_v_step = glam::Vec3::from(target_v) / (count_v as f32);
                    for i in 0..count_u {
                        for j in 0..count_v {
                            let target_point =
                                target_c + (i as f32 * target_u_step) + (j as f32 * target_v_step);
                            rays.push(Ray {
                                origin: ray_origin,
                                direction: (target_point - ray_origin).normalize(),
                                current_ior,
                            });
                        }
                    }
                }
            }
        }

        Scene { objects, rays }
    }
}
