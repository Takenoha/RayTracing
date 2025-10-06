use crate::{aabb::surrounding_box, AABB, HitRecord, Hittable, Ray};
use rand::Rng;

pub struct BVHNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    bbox: AABB,
}

impl BVHNode {
    pub fn new(objects: &mut [Box<dyn Hittable>]) -> Self {
        let axis = rand::thread_rng().gen_range(0..3);
        let comparator = |a: &Box<dyn Hittable>, b: &Box<dyn Hittable>| {
            // Bounding box should exist for all objects used in a BVH.
            let box_a = a.bounding_box().unwrap();
            let box_b = b.bounding_box().unwrap();
            box_a.min[axis].partial_cmp(&box_b.min[axis]).unwrap()
        };

        let (left, right): (Box<dyn Hittable>, Box<dyn Hittable>) = if objects.len() == 1 {
            (objects[0].clone_hittable(), objects[0].clone_hittable())
        } else if objects.len() == 2 {
            if comparator(&objects[0], &objects[1]).is_lt() {
                (objects[0].clone_hittable(), objects[1].clone_hittable())
            } else {
                (objects[1].clone_hittable(), objects[0].clone_hittable())
            }
        } else {
            objects.sort_by(comparator);
            let mid = objects.len() / 2;
            let (left_half, right_half) = objects.split_at_mut(mid);
            (
                Box::new(BVHNode::new(left_half)),
                Box::new(BVHNode::new(right_half)),
            )
        };

        let box_left = left.bounding_box().unwrap();
        let box_right = right.bounding_box().unwrap();
        let bbox = surrounding_box(&box_left, &box_right);

        Self { left, right, bbox }
    }
}

impl Hittable for BVHNode {
    fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }

    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        if !self.bbox.hit(ray, t_min, t_max) {
            return None;
        }

        let hits_left = self.left.intersect_all(ray, t_min, t_max);
        let hits_right = self.right.intersect_all(ray, t_min, t_max);

        match (hits_left, hits_right) {
            (Some(mut l), Some(mut r)) => {
                l.append(&mut r);
                l.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
                Some(l)
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }
}
