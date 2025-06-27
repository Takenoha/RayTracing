// 無限円柱
#[derive(Debug, Clone, Copy)]
struct InfiniteCylinder {
    pub axis_point: Vec3, // 軸上の任意の点
    pub axis_dir: Vec3,   // 軸の方向（正規化されていること）
    pub radius: f32,
    pub material: Material,
}

impl Hittable for InfiniteCylinder {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        // --- 二次方程式の係数 A, B, C を計算 ---
        
        let oc = ray.origin - self.axis_point;

        // ベクトルを軸に平行な成分と垂直な成分に分解する考え方を用いる
        // D_perp = D - (D・V)V  (Vは軸方向ベクトル)
        let d_dot_v = ray.direction.dot(self.axis_dir);
        let d_perp = ray.direction - d_dot_v * self.axis_dir;

        // OC_perp = OC - (OC・V)V
        let oc_dot_v = oc.dot(self.axis_dir);
        let oc_perp = oc - oc_dot_v * self.axis_dir;

        let a = d_perp.length_squared();
        let b = 2.0 * oc_perp.dot(d_perp);
        let c = oc_perp.length_squared() - self.radius * self.radius;

        // --- 二次方程式を解く ---

        // レイが軸とほぼ平行な場合、ヒットしないか常にヒットする。簡単のためミスとする。
        if a.abs() < 1e-6 {
            return None;
        }

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None; // 実数解なし
        }

        let sqrtd = discriminant.sqrt();
        let mut hits = Vec::new();

        // 2つの解を計算
        let t1 = (-b - sqrtd) / (2.0 * a);
        let t2 = (-b + sqrtd) / (2.0 * a);

        for &t in &[t1, t2] {
            if t > t_min && t < t_max {
                let point = ray.origin + t * ray.direction;
                
                // 法線を計算
                // 軸上の最近接点 = P_axis = axis_point + ((P - axis_point)・axis_dir) * axis_dir
                // 法線 N = normalize(P - P_axis)
                let p_minus_a = point - self.axis_point;
                let projection = p_minus_a.dot(self.axis_dir);
                let point_on_axis = self.axis_point + projection * self.axis_dir;
                let outward_normal = (point - point_on_axis).normalize();
                
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let normal = if front_face { outward_normal } else { -outward_normal };

                hits.push(HitRecord { t, point, normal, front_face, material: self.material });
            }
        }
        
        if hits.is_empty() { None } else { Some(hits) }
    }
}
