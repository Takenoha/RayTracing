use glam::Vec3;
use image::{ImageBuffer, Rgb};
use rand::Rng;
use rayon::prelude::*;
use raytracing_config::model::camera_config::CameraConfig;
use raytracing_core::{HitRecord, Material, Ray, Scene, Pdf, CosinePdf, HittablePdf, MixturePdf, reflect, refract, schlick};
use std::path::Path;

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
    let camera = Camera::new(
        camera_config.lookfrom,
        camera_config.lookat,
        camera_config.vup,
        camera_config.vfov,
        width as f32 / height as f32,
    );

    let pixels: Vec<Rgb<u8>> = (0..height)
        .into_par_iter()
        .rev()
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

    let raw_pixels: Vec<u8> = pixels.into_iter().flat_map(|p| p.0).collect();
    let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, raw_pixels).unwrap();

    img_buf.save(output_path).unwrap();
    println!(
        "レンダリングが完了し、'{:?}' に保存されました。",
        output_path
    );
}

// --- Scattering Logic ---

// Returns: (attenuation, scattered_ray) for specular materials, or None for diffuse.
fn scatter_specular(
    material: &Material,
    ray_in: &Ray,
    hit: &HitRecord,
) -> Option<(Vec3, Ray)> {
    match *material {
        Material::Metal { color, fuzz } => {
            let reflected = reflect(ray_in.direction.normalize(), hit.normal);
            let scattered = Ray {
                origin: hit.point,
                direction: reflected + fuzz * rand::thread_rng().gen_range(-1.0..1.0) * Vec3::ONE, // Simplified from random_in_unit_sphere
                current_ior: ray_in.current_ior,
            };
            if scattered.direction.dot(hit.normal) > 0.0 {
                Some((color, scattered))
            } else {
                None
            }
        }
        Material::Glass { color, ior } => {
            let etai_over_etat = if hit.front_face { 1.0 / ior } else { ior };
            let unit_direction = ray_in.direction.normalize();
            let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
            let reflect_prob = schlick(cos_theta, etai_over_etat);

            let direction = if etai_over_etat * sin_theta > 1.0 || rand::thread_rng().r#gen::<f32>() < reflect_prob {
                reflect(unit_direction, hit.normal)
            } else {
                refract(unit_direction, hit.normal, etai_over_etat)
            };

            let scattered = Ray {
                origin: hit.point,
                direction,
                current_ior: if hit.front_face { ior } else { 1.0 },
            };
            Some((color, scattered))
        }
        _ => None,
    }
}

// Returns: (attenuation, pdf) for diffuse materials, or None for specular.
fn scatter_diffuse(
    material: &Material,
    hit: &HitRecord,
) -> Option<(Vec3, CosinePdf)> {
    match *material {
        Material::Diffuse { color } => {
            Some((color, CosinePdf::new(&hit.normal)))
        }
        _ => None,
    }
}


// Recursively traces a ray and determines the color.
fn ray_color(ray: &Ray, scene: &Scene, depth: i32) -> Vec3 {
    if depth <= 0 {
        return Vec3::ZERO;
    }

    let closest_hit = scene
        .world
        .intersect_all(ray, 0.001, f32::INFINITY)
        .and_then(|mut hits| {
            hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
            hits.into_iter().next()
        });

    if let Some(hit) = closest_hit {
        let emitted = hit.material.emitted();

        // Handle specular materials (Metal, Glass)
        if let Some((attenuation, scattered_ray)) = scatter_specular(&hit.material, ray, &hit) {
            return emitted + attenuation * ray_color(&scattered_ray, scene, depth - 1);
        }

        // Handle diffuse materials
        if let Some((attenuation, cosine_pdf)) = scatter_diffuse(&hit.material, &hit) {
            let light_pdf = HittablePdf::new(hit.point, &*scene.lights);
            let mixture_pdf = MixturePdf::new(&light_pdf, &cosine_pdf);
            
            let scattered_direction = mixture_pdf.generate();
            let pdf_val = mixture_pdf.value(scattered_direction);

            let scattered_ray = Ray {
                origin: hit.point,
                direction: scattered_direction,
                current_ior: ray.current_ior,
            };

            // The BRDF for Lambertian is attenuation / PI. The scattering PDF is cos_theta / PI.
            // The final term is (attenuation * scattering_pdf * recursive_color) / mixture_pdf.
            let scattering_pdf = cosine_pdf.value(scattered_direction);

            // Avoid division by zero or negative PDFs
            if pdf_val <= 1e-6 {
                return emitted;
            }

            let recursive_color = ray_color(&scattered_ray, scene, depth - 1);

            return emitted + (attenuation * scattering_pdf * recursive_color) / pdf_val;
        }

        // If it's neither specular nor diffuse (e.g. a light source), just return emitted.
        return emitted;

    } else {
        Vec3::ZERO
    }
}