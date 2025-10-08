use glam::Vec3;
use image::{ImageBuffer, Rgb};
use rand::Rng;
use rayon::prelude::*;
use raytracing_config::model::camera_config::CameraConfig;
use raytracing_core::{Material, Ray, Scene};
use std::path::Path;

// --- Helper Functions for vector math and physics ---

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

fn random_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let p = Vec3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

// --- Camera ---

pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, vup: Vec3, vfov: f32, aspect_ratio: f32) -> Camera {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = lookfrom;
        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - w;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: (self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin)
                .normalize(),
            current_ior: 1.0,
        }
    }
}

// --- Main Rendering Logic ---

/// Renders the scene and saves it to a file.
pub fn render(
    scene: &Scene,
    camera_config: &CameraConfig,
    width: u32,
    height: u32,
    samples_per_pixel: u32,
    max_depth: u32,
    output_path: &Path,
) {
    // Camera
    let camera = Camera::new(
        camera_config.lookfrom,
        camera_config.lookat,
        camera_config.vup,
        camera_config.vfov,
        width as f32 / height as f32,
    );

    // Render using Rayon for parallel processing
    let pixels: Vec<Rgb<u8>> = (0..height)
        .into_par_iter()
        .rev() // Start from the top row
        .flat_map(|j| {
            (0..width)
                .map(|i| {
                    let mut pixel_color = Vec3::ZERO;
                    let mut rng = rand::thread_rng();

                    for _ in 0..samples_per_pixel {
                        let u = (i as f32 + rng.r#gen::<f32>()) / (width - 1) as f32;
                        let v = (j as f32 + rng.r#gen::<f32>()) / (height - 1) as f32;
                        let ray = camera.get_ray(u, v);
                        pixel_color += ray_color(&ray, scene, max_depth as i32);
                    }

                    // Average color and apply gamma correction
                    let scale = 1.0 / samples_per_pixel as f32;
                    let r = (pixel_color.x * scale).sqrt();
                    let g = (pixel_color.y * scale).sqrt();
                    let b = (pixel_color.z * scale).sqrt();

                    Rgb([
                        (256.0 * r.clamp(0.0, 0.999)) as u8,
                        (256.0 * g.clamp(0.0, 0.999)) as u8,
                        (256.0 * b.clamp(0.0, 0.999)) as u8,
                    ])
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Create image buffer from pixel data
    let raw_pixels: Vec<u8> = pixels.into_iter().flat_map(|p| p.0).collect();
    let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, raw_pixels).unwrap();

    // Save the image
    img_buf.save(output_path).unwrap();
    println!(
        "レンダリングが完了し、'{:?}' に保存されました。",
        output_path
    );
}

// Recursively traces a ray and determines the color.
fn ray_color(ray: &Ray, scene: &Scene, depth: i32) -> Vec3 {
    if depth <= 0 {
        return Vec3::ZERO;
    }

    // Find the closest hit by intersecting the ray with the world object (which could be a BVH).
    let closest_hit = scene
        .world
        .intersect_all(ray, 0.001, f32::INFINITY)
        .and_then(|mut hits| {
            // The list of hits should be sorted by t, so we can just take the first one.
            hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
            hits.into_iter().next()
        });

    if let Some(hit) = closest_hit {
        // Calculate emitted light from the material itself
        let emitted = match hit.material {
            Material::Light { color } => color,
            _ => Vec3::ZERO,
        };

        // Calculate scattered light
        let (maybe_scattered, attenuation) = match hit.material {
            Material::Light { .. } => (None, Vec3::ZERO), // Lights emit, but don't scatter

            Material::Diffuse { color } => {
                let scatter_direction = hit.normal + random_in_unit_sphere().normalize();
                let scattered = Ray {
                    origin: hit.point,
                    direction: scatter_direction,
                    current_ior: ray.current_ior,
                };
                (Some(scattered), color)
            }

            Material::Metal { color, fuzz } => {
                let reflected = reflect(ray.direction.normalize(), hit.normal);
                let scattered = Ray {
                    origin: hit.point,
                    direction: reflected + fuzz * random_in_unit_sphere(),
                    current_ior: ray.current_ior,
                };
                // Absorb rays that scatter below the surface
                if scattered.direction.dot(hit.normal) > 0.0 {
                    (Some(scattered), color)
                } else {
                    (None, Vec3::ZERO)
                }
            }

            Material::Glass { color, ior } => {
                let attenuation = color; // Use the material's color for attenuation
                let etai_over_etat = if hit.front_face { 1.0 / ior } else { ior };
                let unit_direction = ray.direction.normalize();
                let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                let reflect_prob = schlick(cos_theta, etai_over_etat);

                let direction = if etai_over_etat * sin_theta > 1.0
                    || rand::thread_rng().r#gen::<f32>() < reflect_prob
                {
                    reflect(unit_direction, hit.normal)
                } else {
                    refract(unit_direction, hit.normal, etai_over_etat)
                };

                let scattered = Ray {
                    origin: hit.point,
                    direction,
                    current_ior: if hit.front_face { ior } else { 1.0 },
                };
                (Some(scattered), attenuation)
            }
            _ => (None, Vec3::ZERO), // Unhandled materials
        };

        if let Some(scattered_ray) = maybe_scattered {
            emitted + attenuation * ray_color(&scattered_ray, scene, depth - 1)
        } else {
            emitted
        }
    } else {
        // If the ray doesn't hit anything, it's black (scene is lit from within)
        Vec3::ZERO
    }
}