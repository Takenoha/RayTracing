use crate::{aabb::surrounding_box, AABB, HitRecord, Hittable, Ray};

// A list of hittable objects
pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let mut all_hits = Vec::new();
        let mut closest_so_far = t_max;

        for object in &self.objects {
            // By reducing t_max with each hit, we can prune the search space
            if let Some(mut hits) = object.intersect_all(ray, t_min, closest_so_far) {
                if let Some(first_hit) = hits.first() {
                    closest_so_far = first_hit.t;
                }
                all_hits.append(&mut hits);
            }
        }

        if all_hits.is_empty() {
            None
        } else {
            all_hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
            Some(all_hits)
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
