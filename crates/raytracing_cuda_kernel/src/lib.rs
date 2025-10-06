#![no_std]
#![feature(abi_ptx)]
#![feature(std_internals)]
#![feature(core_intrinsics)]

use cuda_std::*;
use glam::Vec3;

// --- GPU Data Structures ---

#[derive(Copy, Clone)]
pub enum GpuMaterial {
    Metal { color: Vec3, fuzz: f32 },
    Glass { color: Vec3, ior: f32 },
    HalfMirror { reflectance: f32 },
    Diffuse { color: Vec3 },
    Light { color: Vec3 },
}

#[derive(Copy, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: GpuMaterial,
}

#[derive(Copy, Clone)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: GpuMaterial,
}

#[derive(Copy, Clone)]
pub struct GpuAxisAlignedBox {
    pub min: Vec3,
    pub max: Vec3,
    pub material: GpuMaterial,
}

#[derive(Copy, Clone)]
pub struct GpuInfiniteCylinder {
    pub axis_point: Vec3,
    pub axis_dir: Vec3,
    pub radius: f32,
    pub material: GpuMaterial,
}

#[derive(Copy, Clone)]
pub enum GpuHittable {
    Sphere(Sphere),
    Plane(Plane),
    AxisAlignedBox(GpuAxisAlignedBox),
    InfiniteCylinder(GpuInfiniteCylinder),
}

#[derive(Copy, Clone)]
pub struct GpuCamera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl GpuCamera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, vup: Vec3, vfov: f32, aspect_ratio: f32) -> Self {
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

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    #[inline(always)]
    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: (self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin).normalize(),
            current_ior: 1.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct GpuScene<'a> {
    pub hittables: &'a [GpuHittable],
}

impl<'a> GpuScene<'a> {
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut hit_record = None;
        for hittable in self.hittables {
            if let Some(hit) = hittable.intersect(ray, t_min, closest_so_far) {
                closest_so_far = hit.t;
                hit_record = Some(hit);
            }
        }
        hit_record
    }
}


#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub current_ior: f32,
}

impl Ray {
    #[inline(always)]
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}

#[derive(Copy, Clone)]
pub struct HitRecord {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub front_face: bool,
    pub material: GpuMaterial,
}

impl HitRecord {
    #[inline(always)]
    pub fn face_normal(ray: &Ray, outward_normal: Vec3) -> (Vec3, bool) {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        (normal, front_face)
    }
}

// --- Intersection implementations ---

impl Sphere {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let point = ray.at(t);
        let outward_normal = (point - self.center) / self.radius;
        let (normal, front_face) = HitRecord::face_normal(ray, outward_normal);

        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}

impl Plane {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let denominator = self.normal.dot(ray.direction);
        if denominator.abs() < 1e-6 {
            return None;
        }
        let t = (self.point - ray.origin).dot(self.normal) / denominator;
        if t < t_min || t > t_max {
            return None;
        }
        let point = ray.at(t);
        let (normal, front_face) = HitRecord::face_normal(ray, self.normal);
        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}

impl GpuAxisAlignedBox {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut tmin = t_min;
        let mut tmax = t_max;

        for i in 0..3 {
            let inv_d = 1.0 / ray.direction[i];
            let mut t0 = (self.min[i] - ray.origin[i]) * inv_d;
            let mut t1 = (self.max[i] - ray.origin[i]) * inv_d;

            if inv_d < 0.0 {
                core::mem::swap(&mut t0, &mut t1);
            }

            tmin = tmin.max(t0);
            tmax = tmax.min(t1);

            if tmax <= tmin {
                return None;
            }
        }

        let t = tmin;
        let point = ray.at(t);
        let outward_normal = self.calculate_normal(point);
        let (normal, front_face) = HitRecord::face_normal(ray, outward_normal);

        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }

    fn calculate_normal(&self, point: Vec3) -> Vec3 {
        let epsilon = 1e-4;
        if (point.x - self.min.x).abs() < epsilon { return Vec3::NEG_X; }
        if (point.x - self.max.x).abs() < epsilon { return Vec3::X; }
        if (point.y - self.min.y).abs() < epsilon { return Vec3::NEG_Y; }
        if (point.y - self.max.y).abs() < epsilon { return Vec3::Y; }
        if (point.z - self.min.z).abs() < epsilon { return Vec3::NEG_Z; }
        if (point.z - self.max.z).abs() < epsilon { return Vec3::Z; }
        Vec3::ZERO
    }
}

impl GpuInfiniteCylinder {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.axis_point;
        let d_dot_v = ray.direction.dot(self.axis_dir);
        let d_perp = ray.direction - d_dot_v * self.axis_dir;
        let oc_dot_v = oc.dot(self.axis_dir);
        let oc_perp = oc - oc_dot_v * self.axis_dir;

        let a = d_perp.length_squared();
        let b = 2.0 * oc_perp.dot(d_perp);
        let c = oc_perp.length_squared() - self.radius * self.radius;

        if a.abs() < 1e-6 {
            return None;
        }

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut t = (-b - sqrtd) / (2.0 * a);
        if t < t_min || t > t_max {
            t = (-b + sqrtd) / (2.0 * a);
            if t < t_min || t > t_max {
                return None;
            }
        }

        let point = ray.at(t);
        let p_minus_a = point - self.axis_point;
        let projection = p_minus_a.dot(self.axis_dir);
        let point_on_axis = self.axis_point + projection * self.axis_dir;
        let outward_normal = (point - point_on_axis).normalize();
        let (normal, front_face) = HitRecord::face_normal(ray, outward_normal);

        Some(HitRecord {
            t,
            point,
            normal,
            front_face,
            material: self.material,
        })
    }
}


impl GpuHittable {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            GpuHittable::Sphere(s) => s.intersect(ray, t_min, t_max),
            GpuHittable::Plane(p) => p.intersect(ray, t_min, t_max),
            GpuHittable::AxisAlignedBox(b) => b.intersect(ray, t_min, t_max),
            GpuHittable::InfiniteCylinder(c) => c.intersect(ray, t_min, t_max),
        }
    }
}

// --- Helper Functions for Kernel ---

#[inline(always)]
fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

#[inline(always)]
fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

#[inline(always)]
fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

// LCG random number generator for the GPU
#[inline(always)]
fn lcg(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    (*seed as f32) / (u32::MAX as f32)
}

#[inline(always)]
fn random_in_unit_sphere(seed: &mut u32) -> Vec3 {
    loop {
        let p = Vec3::new(lcg(seed) * 2.0 - 1.0, lcg(seed) * 2.0 - 1.0, lcg(seed) * 2.0 - 1.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

// --- Main Kernel Logic ---

fn kernel_ray_color(mut ray: Ray, scene: &GpuScene, max_depth: i32, seed: &mut u32) -> Vec3 {
    let mut color = Vec3::ONE;
    for _ in 0..max_depth {
        if let Some(hit) = scene.hit(&ray, 0.001, f32::INFINITY) {
            let emitted = match hit.material {
                GpuMaterial::Light { color } => color,
                _ => Vec3::ZERO,
            };

            let (maybe_scattered, attenuation) = match hit.material {
                GpuMaterial::Light { .. } => (None, Vec3::ZERO),
                GpuMaterial::Diffuse { color } => {
                    let scatter_direction = hit.normal + random_in_unit_sphere(seed).normalize();
                    let scattered = Ray {
                        origin: hit.point,
                        direction: scatter_direction,
                        current_ior: ray.current_ior,
                    };
                    (Some(scattered), color)
                }
                GpuMaterial::Metal { color, fuzz } => {
                    let reflected = reflect(ray.direction.normalize(), hit.normal);
                    let scattered = Ray {
                        origin: hit.point,
                        direction: reflected + fuzz * random_in_unit_sphere(seed),
                        current_ior: ray.current_ior,
                    };
                    if scattered.direction.dot(hit.normal) > 0.0 {
                        (Some(scattered), color)
                    } else {
                        (None, Vec3::ZERO)
                    }
                }
                GpuMaterial::Glass { color: _, ior } => {
                    let etai_over_etat = if hit.front_face { 1.0 / ior } else { ior };
                    let unit_direction = ray.direction.normalize();
                    let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
                    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                    let reflect_prob = schlick(cos_theta, etai_over_etat);

                    let direction = if etai_over_etat * sin_theta > 1.0 || lcg(seed) < reflect_prob {
                        reflect(unit_direction, hit.normal)
                    } else {
                        refract(unit_direction, hit.normal, etai_over_etat)
                    };
                    let scattered = Ray {
                        origin: hit.point,
                        direction,
                        current_ior: if hit.front_face { ior } else { 1.0 },
                    };
                    (Some(scattered), Vec3::ONE) // Glass is perfectly transparent
                }
                 _ => (None, Vec3::ZERO),
            };

            color *= attenuation;
            if let Some(scattered) = maybe_scattered {
                ray = scattered;
            } else {
                return emitted;
            }
        } else {
            return Vec3::ZERO; // Background
        }
    }
    Vec3::ZERO
}

#[no_mangle]
pub unsafe extern "ptx-kernel" fn render_kernel(
    hittables: *const GpuHittable,
    hittable_count: u32,
    camera: *const GpuCamera,
    output: *mut Vec3,
    width: u32,
    height: u32,
    samples_per_pixel: u32,
    max_depth: i32,
) {
    let thread_idx = thread::idx().x;
    let block_idx = block::idx().x;
    let block_dim = block::dim().x;

    let i = block_idx * block_dim + thread_idx;
    if i >= width * height {
        return;
    }

    let x = i % width;
    let y = i / width;

    let scene = GpuScene {
        hittables: core::slice::from_raw_parts(hittables, hittable_count as usize),
    };
    let camera = &*camera;

    let mut pixel_color = Vec3::ZERO;
    let mut seed = (i as u32).wrapping_mul(123456789);

    for _ in 0..samples_per_pixel {
        let u = (x as f32 + lcg(&mut seed)) / (width - 1) as f32;
        let v = (y as f32 + lcg(&mut seed)) / (height - 1) as f32;
        let ray = camera.get_ray(u, v);
        pixel_color += kernel_ray_color(ray, &scene, max_depth, &mut seed);
    }

    *output.add(i as usize) = pixel_color / samples_per_pixel as f32;
}