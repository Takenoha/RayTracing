// CSGオブジェクト
struct CSGObject {
    pub left: Box<dyn Hittable>,
    pub right: Box<dyn Hittable>,
    pub operation: CsgOperation,
}
impl Hittable for CSGObject {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        // 1. 左右の子オブジェクトとの全ての交点を取得
        let hits_left = self.left.intersect_all(ray, t_min, t_max).unwrap_or_default();
        let hits_right = self.right.intersect_all(ray, t_min, t_max).unwrap_or_default();

        // 2. 全てのヒットを一つのリストにまとめ、tでソート
        let mut all_hits = hits_left.clone();
        all_hits.extend(hits_right.clone());
        all_hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

        let mut result_hits = Vec::new();
        
        // 3. 演算の種類に応じたフィルタリング処理
        let mut in_left = false;
        let mut in_right = false;

        for hit in &all_hits {
            // このヒットがleft/rightどちらの物か判定
            let hit_is_on_left = hits_left.iter().any(|h| (h.t - hit.t).abs() < 1e-6);

            // 演算前の状態を保存
            let was_inside = match self.operation {
                CsgOperation::Union => in_left || in_right,
                CsgOperation::Intersection => in_left && in_right,
                CsgOperation::Difference => in_left && !in_right,
            };

            // 内外状態を更新
            if hit_is_on_left {
                in_left = !in_left;
            } else {
                in_right = !in_right;
            }

            // 演算後の状態を計算
            let is_inside = match self.operation {
                CsgOperation::Union => in_left || in_right,
                CsgOperation::Intersection => in_left && in_right,
                CsgOperation::Difference => in_left && !in_right,
            };
            
            // 状態が変化した（＝CSGオブジェクトの表面を通過した）なら、そのヒットは有効
            if was_inside != is_inside {
                // Differenceの場合、rightオブジェクトの法線は反転させる必要がある
                if self.operation == CsgOperation::Difference && !hit_is_on_left {
                    let mut inverted_hit = *hit;
                    inverted_hit.normal = -hit.normal;
                    inverted_hit.front_face = !hit.front_face;
                    result_hits.push(inverted_hit);
                } else {
                    result_hits.push(*hit);
                }
            }
        }

        if result_hits.is_empty() {
            None
        } else {
            Some(result_hits)
        }
    }
}