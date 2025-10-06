use glam::Vec3;
use raytracing_core::{Hittable, Ray, Scene};
use std::fs::File;
use std::io::{self, BufWriter, Write};

// A simple hittable trait for the renderer, similar to the one in raytracing_core
// but adapted for the renderer's needs.
trait RenderHittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo>;
}

// Information about a ray-object intersection.
struct HitInfo<'a> {
    t: f32,
    p: Vec3,
    normal: Vec3,
    material: &'a dyn Material,
}

// A simple material trait.
trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitInfo) -> Option<(Vec3, Ray)>;
}

// Simple diffuse material.
struct Lambertian {
    albedo: Vec3,
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitInfo) -> Option<(Vec3, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::new(1.0, 1.0, 1.0).normalize();
        if scatter_direction.length_squared() < 1e-8 {
            scatter_direction = rec.normal;
        }
        let scattered = Ray {
            origin: rec.p,
            direction: scatter_direction,
            current_ior: 1.0, // Assuming air
        };
        Some((self.albedo, scattered))
    }
}

// The main render function.
pub fn render(scene: &Scene, _results: &Vec<Vec<Vec3>>) -> io::Result<()> {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f32 / aspect_ratio) as i32;

    // Camera
    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin = Vec3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    // Render
    let file = File::create("output.ppm")?;
    let mut out = BufWriter::new(file);

    writeln!(out, "P3\n{} {}\n255", image_width, image_height)?;

    for j in (0..image_height).rev() {
        eprintln!("\rScanlines remaining: {} ", j);
        for i in 0..image_width {
            let u = i as f32 / (image_width - 1) as f32;
            let v = j as f32 / (image_height - 1) as f32;
            let r = Ray {
                origin,
                direction: lower_left_corner + u * horizontal + v * vertical - origin,
                current_ior: 1.0,
            };
            let pixel_color = ray_color(&r, scene, 50);
            write_color(&mut out, pixel_color)?;
        }
    }
    eprintln!("\nDone.");

    Ok(())
}

fn ray_color(r: &Ray, scene: &Scene, depth: i32) -> Vec3 {
    if depth <= 0 {
        return Vec3::ZERO;
    }

    let mut closest_hit_record: Option<raytracing_core::HitRecord> = None;
    let mut t_closest = f32::INFINITY;

    for object in &scene.objects {
        if let Some(hits) = object.intersect_all(r, 0.001, t_closest) {
            if let Some(first_hit) = hits.first() {
                if first_hit.t < t_closest {
                    t_closest = first_hit.t;
                    closest_hit_record = Some(*first_hit);
                }
            }
        }
    }

    if let Some(rec) = closest_hit_record {
        // A simple visualization of normals
        return 0.5 * (rec.normal + Vec3::new(1.0, 1.0, 1.0));
    }

    // Background
    let unit_direction = r.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Vec3::new(1.0, 1.0, 1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

fn write_color<W: Write>(out: &mut W, pixel_color: Vec3) -> io::Result<()> {
    let r = (255.999 * pixel_color.x) as i32;
    let g = (255.999 * pixel_color.y) as i32;
    let b = (255.999 * pixel_color.z) as i32;
    writeln!(out, "{} {} {}", r, g, b)
}