use crate::Hittable;
use glam::Vec3;
use rand::Rng;
use std::f32::consts::PI;

// Orthonormal Basis for creating coordinate systems
#[derive(Debug, Clone, Copy)]
pub struct Onb {
    pub axis: [Vec3; 3],
}

impl Onb {
    pub fn build_from_w(n: &Vec3) -> Self {
        let w = n.normalize();
        let a = if w.x.abs() > 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);
        Self { axis: [u, v, w] }
    }

    pub fn local(&self, a: Vec3) -> Vec3 {
        a.x * self.axis[0] + a.y * self.axis[1] + a.z * self.axis[2]
    }
}

pub trait Pdf: Sync + Send {
    fn value(&self, direction: Vec3) -> f32;
    fn generate(&self) -> Vec3;
}

// --- Cosine PDF for diffuse materials ---

pub struct CosinePdf {
    onb: Onb,
}

impl CosinePdf {
    pub fn new(w: &Vec3) -> Self {
        Self { onb: Onb::build_from_w(w) }
    }
}

fn random_cosine_direction() -> Vec3 {
    let mut rng = rand::thread_rng();
    let r1 = rng.r#gen::<f32>();
    let r2 = rng.r#gen::<f32>();
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * PI * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    Vec3::new(x, y, z)
}

impl Pdf for CosinePdf {
    fn value(&self, direction: Vec3) -> f32 {
        let cos = direction.normalize().dot(self.onb.axis[2]);
        if cos <= 0.0 {
            0.0
        } else {
            cos / PI
        }
    }

    fn generate(&self) -> Vec3 {
        self.onb.local(random_cosine_direction())
    }
}

// --- Hittable PDF for light sampling ---

pub struct HittablePdf<'a> {
    origin: Vec3,
    hittable: &'a dyn Hittable,
}

impl<'a> HittablePdf<'a> {
    pub fn new(origin: Vec3, hittable: &'a dyn Hittable) -> Self {
        Self { origin, hittable }
    }
}

impl<'a> Pdf for HittablePdf<'a> {
    fn value(&self, direction: Vec3) -> f32 {
        self.hittable.pdf_value(self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.hittable.random(self.origin)
    }
}

// --- Mixture PDF to combine two PDFs ---

pub struct MixturePdf<'a> {
    p: [&'a dyn Pdf; 2],
}

impl<'a> MixturePdf<'a> {
    pub fn new(p0: &'a dyn Pdf, p1: &'a dyn Pdf) -> Self {
        Self { p: [p0, p1] }
    }
}

impl<'a> Pdf for MixturePdf<'a> {
    fn value(&self, direction: Vec3) -> f32 {
        0.5 * self.p[0].value(direction) + 0.5 * self.p[1].value(direction)
    }

    fn generate(&self) -> Vec3 {
        if rand::thread_rng().r#gen::<f32>() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}