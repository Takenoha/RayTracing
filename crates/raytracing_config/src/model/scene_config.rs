use super::material_config::MaterialConfig;
use super::object_config::ObjectConfig;
use super::object_generator_config::{ObjectGeneratorConfig, RayGeneratorConfig};
use super::ray_config::RayConfig;
use raytracing_core::{BVHNode, Hittable, HittableList, Ray, Scene};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct SceneConfig {
    #[serde(default)]
    pub objects: Vec<ObjectConfig>,
    #[serde(default)]
    pub object_generators: Vec<ObjectGeneratorConfig>,
    #[serde(default)]
    pub rays: Vec<RayConfig>,
    #[serde(default)]
    pub ray_generators: Vec<RayGeneratorConfig>,
}

impl From<SceneConfig> for Scene {
    fn from(config: SceneConfig) -> Self {
        // 1. Generate all ObjectConfig instances
        let mut all_object_configs: Vec<ObjectConfig> = config.objects;

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
                    let start_pos = glam::Vec3::from(position_start);
                    let x_step = glam::Vec3::from(step_x);
                    let z_step = glam::Vec3::from(step_z);
                    for i in 0..count_x {
                        for j in 0..count_z {
                            let pos = start_pos + (i as f32 * x_step) + (j as f32 * z_step);
                            let mut obj = template.clone();
                            obj.transform.position = pos.to_array();
                            all_object_configs.push(obj);
                        }
                    }
                }
            }
        }

        // 2. Partition configs into lights and non-lights, then convert to Hittable
        let mut light_objects: Vec<Box<dyn Hittable>> = Vec::new();
        let mut all_world_objects: Vec<Box<dyn Hittable>> = Vec::new();

        for obj_config in all_object_configs {
            let is_light = matches!(obj_config.material, MaterialConfig::Light { .. });
            let hittable: Box<dyn Hittable> = obj_config.into();

            if is_light {
                light_objects.push(hittable.clone_hittable());
            }
            all_world_objects.push(hittable);
        }

        // 3. Build the world from all objects
        let (mut bounded_objects, unbounded_objects): (Vec<_>, Vec<_>) = all_world_objects
            .into_iter()
            .partition(|obj| obj.bounding_box().is_some());

        let mut world_list = HittableList::new();
        for obj in unbounded_objects {
            world_list.add(obj);
        }

        if !bounded_objects.is_empty() {
            let bvh_node = BVHNode::new(&mut bounded_objects);
            world_list.add(Box::new(bvh_node));
        }

        // 4. Build the lights list
        let mut lights_list = HittableList::new();
        for light in light_objects {
            lights_list.add(light);
        }

        // 5. Generate all rays (same as before)
        let mut rays: Vec<Ray> = config.rays.into_iter().map(Into::into).collect();
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
                            let dir = (target_point - ray_origin).normalize();
                            rays.push(Ray {
                                origin: ray_origin,
                                direction: dir,
                                current_ior,
                            });
                        }
                    }
                }
            }
        }

        // 6. Create the final scene
        Scene {
            world: Box::new(world_list),
            lights: Box::new(lights_list),
            rays,
        }
    }
}
