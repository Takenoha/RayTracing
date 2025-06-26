use glam::Vec3;
use crate::{Hittable, HitRecord, Ray, Material}; // main.rsから移動させる共通定義をインポート

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub point: Vec3, // 平面上の任意の点
    pub normal: Vec3, // 平面の法線
    pub material: Material,
}
impl Hittable for Plane {
    // 全ての交点をリストで返すメソッド
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        // 分母 self.normal.dot(ray.direction) を計算
        let denom = self.normal.dot(ray.direction);

        // レイが平面と平行な場合、分母がほぼ0になるので衝突しない
        if denom.abs() < 1e-6 {
            return None;
        }

        // 交点までの距離 t を計算
        let t = (self.point - ray.origin).dot(self.normal) / denom;

        // tが有効範囲外なら衝突しない
        if t < t_min || t_max < t {
            return None;
        }

        // --- 有効な交点が見つかった場合の処理 ---

        // 衝突点の座標を計算
        let point = ray.origin + t * ray.direction;
        
        // レイが表面から当たったか、裏面から当たったかを判定
        let front_face = ray.direction.dot(self.normal) < 0.0;
        // 法線ベクトルは常にレイと向かい合うように調整
        let normal = if front_face { self.normal } else { -self.normal };
        
        let hit_record = HitRecord { t, point, normal, front_face ,material: self.material};
        
        // ★★★ 変更点 ★★★
        // 単一のHitRecordを、要素が1つのVec（ベクタ）に入れてSomeで返す
        Some(vec![hit_record])
    }
}