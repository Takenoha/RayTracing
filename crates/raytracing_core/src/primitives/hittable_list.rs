use crate::{AABB, HitRecord, Hittable, Ray, primitives::aabb::surrounding_box};

// A list of hittable objects
pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let mut hits = Vec::new();
        for obj in &self.objects {
            // obj.intersect_all が None を返す可能性があるので安全に扱う
            let child_hits = obj.intersect_all(ray, t_min, t_max).unwrap_or_default();
            hits.extend(child_hits);
        }
        // non-finite な t を落とす（念のため）
        let before = hits.len();
        hits.retain(|h| {
            if !h.t.is_finite() {
                eprintln!("HittableList: dropped non-finite hit.t = {:?}", h.t);
                false
            } else {
                true
            }
        });
        if hits.is_empty() {
            None
        } else {
            // 必要ならここで t でソート（安定ソート）
            hits.sort_by(|a, b| a.t.total_cmp(&b.t));
            if hits.len() != before {
                eprintln!(
                    "HittableList: removed {} invalid hit(s)",
                    before - hits.len()
                );
            }
            Some(hits)
        }
    }

    fn bounding_box(&self) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }

        let mut output_box: Option<AABB> = None;

        for object in &self.objects {
            if let Some(temp_box) = object.bounding_box() {
                output_box = match output_box {
                    Some(b) => Some(surrounding_box(&b, &temp_box)),
                    None => Some(temp_box),
                };
            } else {
                // If any object in the list is unbounded, the list itself is unbounded.
                return None;
            }
        }
        output_box
    }

    fn clone_hittable(&self) -> Box<dyn Hittable> {
        let mut new_list = HittableList::new();
        for object in &self.objects {
            new_list.add(object.clone_hittable());
        }
        Box::new(new_list)
    }
}
