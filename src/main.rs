use std::error::Error;

// 3Dベクトルを扱うためにglamクレートのVec3をインポート
use glam::{Vec3,Vec4,Mat4};

// 光線を表す構造体
// origin: 始点, direction: 方向
struct Ray {
    origin: Vec3,
    direction: Vec3,
    current_ior: f32,
}

// 衝突（ヒット）に関する情報をまとめる構造体
#[derive(Debug, Clone, Copy)]
struct HitRecord {
    t: f32,
    point: Vec3,
    normal: Vec3,
    front_face: bool,
    material: Material, // ★★★ この行を追加 ★★★
}

trait Hittable: Send + Sync { // Send + Sync は並列処理のためのマーカー（今は気にしなくてOK）
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>>;
}

// 他のHittableオブジェクトに変換を適用するためのラッパー
struct Transform {
    object: Box<dyn Hittable>,
    transform: Mat4,         // ローカル空間 -> ワールド空間への変換
    inverse_transform: Mat4, // ワールド空間 -> ローカル空間への変換
}

impl Transform {
    pub fn new(object: Box<dyn Hittable>, transform: Mat4) -> Self {
        Self {
            object,
            transform,
            inverse_transform: transform.inverse(), // 逆行列も保持
        }
    }
}

// TransformのためのHittable実装を追加
impl Hittable for Transform {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        // 1. レイをワールド空間からオブジェクトのローカル空間へ逆変換
        let local_ray_origin = self.inverse_transform.transform_point3(ray.origin);
        let local_ray_direction = self.inverse_transform.transform_vector3(ray.direction);
        let local_ray = Ray {
            origin: local_ray_origin,
            direction: local_ray_direction,
            current_ior: ray.current_ior,
        };

        // 2. ローカル空間で、包み込んだオブジェクトとの交差判定を行う
        if let Some(local_hits) = self.object.intersect_all(&local_ray, t_min, t_max) {
            // 3. 結果をローカル空間からワールド空間へ変換して返す
            let world_hits = local_hits.into_iter().map(|mut hit| {
                hit.point = self.transform.transform_point3(hit.point);
                // 法線ベクトルの変換は、逆行列の転置行列をかけるのが数学的に正しい
                hit.normal = self.inverse_transform.transpose().transform_vector3(hit.normal).normalize();
                hit
            }).collect();
            Some(world_hits)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Material {
    Mirror,
    Glass { ior: f32 },
}
// ============== 3D用の物理計算関数 ==============

// 反射ベクトルを計算
fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

// 屈折ベクトルを計算（全反射の可能性も考慮）
fn refract(incident: Vec3, normal: Vec3, ior_ratio: f32) -> Option<Vec3> {
    let cos_theta = (-incident).dot(normal).min(1.0);
    let sin_theta_squared = 1.0 - cos_theta * cos_theta;

    if ior_ratio * ior_ratio * sin_theta_squared > 1.0 {
        return None; // 全反射
    }

    let perp = ior_ratio * (incident + cos_theta * normal);
    let parallel = -(1.0 - perp.length_squared()).abs().sqrt() * normal;
    
    Some((perp + parallel).normalize())
}

// ============== 3D形状の実装 ==============

// 球
#[derive(Debug, Clone, Copy)]
struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Hittable for Sphere {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 { return None; }
        
        let sqrtd = discriminant.sqrt();
        let mut hits = Vec::new();

        // 1つ目の解
        let t1 = (-half_b - sqrtd) / a;
        if t1 > t_min && t1 < t_max {
            // ... (point, normal, front_faceの計算は変更なし) ...
            let point = ray.origin + t1 * ray.direction;
            let outward_normal = (point - self.center) / self.radius;
            let front_face = ray.direction.dot(outward_normal) < 0.0;
            let normal = if front_face { outward_normal } else { -outward_normal };
            
            hits.push(HitRecord { t: t1, point, normal, front_face, material: self.material }); // ★ materialを追加
        }

        // 2つ目の解
        if discriminant > 1e-6 {
            let t2 = (-half_b + sqrtd) / a;
            if t2 > t_min && t2 < t_max {
                // ... (point, normal, front_faceの計算は変更なし) ...
                let point = ray.origin + t2 * ray.direction;
                let outward_normal = (point - self.center) / self.radius;
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let normal = if front_face { outward_normal } else { -outward_normal };

                hits.push(HitRecord { t: t2, point, normal, front_face, material: self.material }); // ★ materialを追加
            }
        }
        
        if hits.is_empty() { None } else { Some(hits) }
    }
}


// 無限平面
#[derive(Debug, Clone, Copy)]
struct Plane {
    point: Vec3, // 平面上の任意の点
    normal: Vec3, // 平面の法線
    material: Material,
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

// 無限円柱
#[derive(Debug, Clone, Copy)]
struct InfiniteCylinder {
    axis_point: Vec3, // 軸上の任意の点
    axis_dir: Vec3,   // 軸の方向（正規化されていること）
    radius: f32,
    material: Material,
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

// 無限円錐
#[derive(Debug, Clone, Copy)]
struct InfiniteCone {
    vertex: Vec3,     // 円錐の頂点
    axis_dir: Vec3,   // 軸の方向（正規化されていること）
    cos_angle_sq: f32, // 開き角度のコサインの2乗 (cos^2(α))
    material: Material,
}

impl InfiniteCone {
    // 角度(ラジアン)からcos^2(α)を計算するコンストラクタ
    pub fn new(vertex: Vec3, axis_dir: Vec3, angle_rad: f32, material: Material) -> Self {
        Self {
            vertex,
            axis_dir: axis_dir.normalize(),
            cos_angle_sq: angle_rad.cos().powi(2),
            material,
        }
    }
}

// InfiniteCone のための Hittable 実装
impl Hittable for InfiniteCone {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let co = ray.origin - self.vertex; // 頂点からレイの始点へのベクトル

        let d_dot_v = ray.direction.dot(self.axis_dir);
        let co_dot_v = co.dot(self.axis_dir);
        
        // 二次方程式の係数 A, B, C を計算
        // A = (D・V)^2 - cos^2(α)
        // B = 2 * [ (D・V)(CO・V) - (D・CO)cos^2(α) ]
        // C = (CO・V)^2 - (CO・CO)cos^2(α)
        // (Dは正規化済みなので D・D = 1 と仮定)
        let a = d_dot_v.powi(2) - self.cos_angle_sq;
        let b = 2.0 * (d_dot_v * co_dot_v - ray.direction.dot(co) * self.cos_angle_sq);
        let c = co_dot_v.powi(2) - co.length_squared() * self.cos_angle_sq;

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
                // N = normalize( (P-V) - (1+tan^2(α)) * ((P-V)・V) * V ) を元に計算
                // より単純な勾配法 N = normalize( (PV・v)v - cos²(α)PV ) を使う
                let pv = point - self.vertex;
                let m = pv.dot(self.axis_dir);
                let outward_normal = (m * self.axis_dir - pv * self.cos_angle_sq).normalize();
                
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let normal = if front_face { outward_normal } else { -outward_normal };

                hits.push(HitRecord { t, point, normal, front_face, material: self.material });
            }
        }

        if hits.is_empty() { None } else { Some(hits) }
    }
}

// 軸並行な直方体 (AABB) 対角の座標を指定
#[derive(Debug, Clone, Copy)]
struct AxisAlignedBox {
    min: Vec3, // 3つの軸の最小座標 (x_min, y_min, z_min)
    max: Vec3, // 3つの軸の最大座標 (x_max, y_max, z_max)
    material: Material,
}
// AxisAlignedBox のための Hittable 実装
impl Hittable for AxisAlignedBox {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        let mut tmin = t_min;
        let mut tmax = t_max;

        // 各軸 (X, Y, Z) に対してSlab Testを実行
        for i in 0..3 {
            // レイの方向の逆数。ゼロ除算を避ける
            let inv_d = 1.0 / ray.direction[i];
            let mut t0 = (self.min[i] - ray.origin[i]) * inv_d;
            let mut t1 = (self.max[i] - ray.origin[i]) * inv_d;
            
            // レイの進行方向に応じて、t0とt1（スラブへの入口と出口）を入れ替える
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            // これまで計算された全体の区間と、現在の軸の区間の共通部分を求める
            tmin = tmin.max(t0);
            tmax = tmax.min(t1);

            // 共通区間がなくなれば、ヒットしない
            if tmax <= tmin {
                return None;
            }
        }

        // --- 有効な交差区間 [tmin, tmax] が見つかった ---
        let mut hits = Vec::new();

        // 最初のヒット (入口)
        let point1 = ray.origin + tmin * ray.direction;
        let normal1 = self.calculate_normal(point1);
        hits.push(HitRecord {
            t: tmin,
            point: point1,
            normal: normal1,
            front_face: ray.direction.dot(normal1) < 0.0,
            material: self.material,
        });

        // 2番目のヒット (出口)
        let point2 = ray.origin + tmax * ray.direction;
        let normal2 = -self.calculate_normal(point2); // 出口の法線は内側を向く
        hits.push(HitRecord {
            t: tmax,
            point: point2,
            normal: normal2,
            front_face: ray.direction.dot(normal2) < 0.0,
            material: self.material,
        });
        
        Some(hits)
    }
}

// AABBのためのヘルパーメソッド
impl AxisAlignedBox {
    // 衝突点から、どの面の法線かを計算する
    fn calculate_normal(&self, point: Vec3) -> Vec3 {
        let epsilon = 1e-4;
        let p_minus_min = point - self.min;
        let p_minus_max = point - self.max;
        
        if p_minus_min.x.abs() < epsilon { return Vec3::NEG_X; }
        if p_minus_max.x.abs() < epsilon { return Vec3::X; }
        if p_minus_min.y.abs() < epsilon { return Vec3::NEG_Y; }
        if p_minus_max.y.abs() < epsilon { return Vec3::Y; }
        if p_minus_min.z.abs() < epsilon { return Vec3::NEG_Z; }
        if p_minus_max.z.abs() < epsilon { return Vec3::Z; }
        
        Vec3::ZERO // 本来は到達しない
    }
}

//レンズプリミティブ
struct Lens {
    csg_object: Box<dyn Hittable>,
}
// Lens構造体の実装ブロックを追加
impl Lens {
    pub fn new(
        center_thickness: f32,
        diameter: f32,
        r1: f32,
        r2: f32,
        material: Material,
    ) -> Self {
        // --- レンズの形状をCSGで組み立てる ---

        // 1. 2つの球面を定義する
        // レンズの中心を原点(0,0,0)に、光軸をZ軸に沿って配置する
        let half_thickness = center_thickness / 2.0;

        // 第1面 (光がZの負方向から来るとして、z = -half_thickness に頂点)
        let s1 = if r1.is_finite() {
            let center1 = Vec3::new(0.0, 0.0, -half_thickness + r1);
            Box::new(Sphere { center: center1, radius: r1.abs(), material }) as Box<dyn Hittable>
        } else {
            // 曲率半径が無限大なら、平面
            Box::new(Plane { point: Vec3::new(0.0, 0.0, -half_thickness), normal: Vec3::Z, material }) as Box<dyn Hittable>
        };

        // 第2面 (z = +half_thickness に頂点)
        let s2 = if r2.is_finite() {
            let center2 = Vec3::new(0.0, 0.0, half_thickness + r2);
            Box::new(Sphere { center: center2, radius: r2.abs(), material }) as Box<dyn Hittable>
        } else {
            Box::new(Plane { point: Vec3::new(0.0, 0.0, half_thickness), normal: Vec3::NEG_Z, material }) as Box<dyn Hittable>
        };

        // 2つの球面の積集合をとる
        let infinite_lens = Box::new(CSGObject {
            left: s1,
            right: s2,
            operation: CsgOperation::Intersection,
        });

        // 2. レンズの直径を制限する円柱を定義
        let aperture_cylinder = Box::new(InfiniteCylinder {
            axis_point: Vec3::ZERO,
            axis_dir: Vec3::Z, // 光軸
            radius: diameter / 2.0,
            material, // 材質はダミー
        });

        // 3. 無限レンズと円柱の積集合をとって、最終的なレンズ形状を完成させる
        let final_lens = Box::new(CSGObject {
            left: infinite_lens,
            right: aperture_cylinder,
            operation: CsgOperation::Intersection,
        });

        Lens { csg_object: final_lens }
    }
}
// LensのためのHittable実装を追加
impl Hittable for Lens {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        self.csg_object.intersect_all(ray, t_min, t_max)
    }
}
//ウェッジ
struct Wedge {
    csg_object: Box<dyn Hittable>,
}
// Wedge構造体の実装ブロックを追加
impl Wedge {
    pub fn new(size: Vec3, wedge_angle_rad: f32, material: Material) -> Self {
        let width = size.x;
        let height = size.y;
        let half_depth = size.z / 2.0;

        // --- 5枚の平面を定義 ---
        let p1 = Box::new(Plane { // 底面 (y >= 0)
            point: Vec3::ZERO,
            normal: Vec3::Y,
            material,
        }) as Box<dyn Hittable>;

        let p2 = Box::new(Plane { // 垂直面 (x >= 0)
            point: Vec3::ZERO,
            normal: Vec3::X,
            material,
        }) as Box<dyn Hittable>;
        
        // 傾斜面
        let angle_cos = wedge_angle_rad.cos();
        let angle_sin = wedge_angle_rad.sin();
        let p3 = Box::new(Plane {
            point: Vec3::ZERO,
            normal: Vec3::new(-angle_sin, angle_cos, 0.0), // 法線で傾きを表現
            material,
        }) as Box<dyn Hittable>;

        let p4 = Box::new(Plane { // 前面キャップ (z <= half_depth)
            point: Vec3::new(0.0, 0.0, half_depth),
            normal: Vec3::NEG_Z, // 法線を反転させることで、zが小さい側が「内側」になる
            material,
        }) as Box<dyn Hittable>;
        
        let p5 = Box::new(Plane { // 背面キャップ (z >= -half_depth)
            point: Vec3::new(0.0, 0.0, -half_depth),
            normal: Vec3::Z,
            material,
        }) as Box<dyn Hittable>;

        // --- CSGの積集合で5枚の平面を組み合わせる ---
        let csg1 = Box::new(CSGObject { left: p1, right: p2, operation: CsgOperation::Intersection });
        let csg2 = Box::new(CSGObject { left: csg1, right: p3, operation: CsgOperation::Intersection });
        let csg3 = Box::new(CSGObject { left: csg2, right: p4, operation: CsgOperation::Intersection });
        let final_wedge = Box::new(CSGObject { left: csg3, right: p5, operation: CsgOperation::Intersection });

        Wedge { csg_object: final_wedge }
    }
}
// WedgeのためのHittable実装を追加
impl Hittable for Wedge {
    fn intersect_all(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Vec<HitRecord>> {
        self.csg_object.intersect_all(ray, t_min, t_max)
    }
}

// ブーリアン演算の種類
#[derive(Debug, Clone, Copy ,PartialEq)]
enum CsgOperation {
    Union,        // 和集合
    Intersection, // 積集合
    Difference,   // 差集合
}
// CSGオブジェクト
struct CSGObject {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    operation: CsgOperation,
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

// --- ここからが2Dプロトタイピング用のコード ---
/* 
// 2Dの光線
#[derive(Debug, Clone, Copy)]
struct Ray2D {
    origin: Vec2,
    direction: Vec2,
    current_ior: f32, // ★★★ この行を追加 ★★★
}
// 2Dの円（レンズや曲面鏡の断面）
struct Circle {
    center: Vec2,
    radius: f32,
    material: Material, // 材質情報
}

// 2Dの線分（平面鏡の断面）
struct LineSegment {
    p1: Vec2,
    p2: Vec2,
    material: Material,
}

// 2Dの衝突情報
struct Hit2D {
    t: f32,
    point: Vec2,
    normal: Vec2,
}

// 材質を定義するenum
#[derive(Debug, Clone, Copy)]
enum Material {
    Mirror,
    Glass { ior: f32 }, // ior: Index of Refraction (屈折率)
}

// --- 必要なトレイトとヘルパー関数 ---

trait Hittable2D {
    fn intersect(&self, ray: &Ray2D) -> Option<Hit2D>;
    fn get_material(&self) -> Material;
}

impl Hittable2D for Circle {
    fn intersect(&self, ray: &Ray2D) -> Option<Hit2D> { intersect_circle(ray, self) }
    fn get_material(&self) -> Material { self.material }
}

impl Hittable2D for LineSegment {
    fn intersect(&self, ray: &Ray2D) -> Option<Hit2D> { intersect_line(ray, self) }
    fn get_material(&self) -> Material { self.material }
}

// 2D反射ベクトルを計算する関数
fn reflect(incident: Vec2, normal: Vec2) -> Vec2 {
    incident - 2.0 * incident.dot(normal) * normal
}
// 2D屈折ベクトルを計算する関数
// incident: 入射ベクトル, normal: 法線ベクトル, ior_ratio: 屈折率の比 (n1 / n2)
fn refract(incident: Vec2, normal: Vec2, ior_ratio: f32) -> Option<Vec2> {
    let cos_theta = (-incident).dot(normal).min(1.0);
    let sin_theta_squared = 1.0 - cos_theta * cos_theta;

    // 全反射の条件をチェック
    // 屈折率の比の2乗 * sin^2(theta) > 1.0 なら全反射
    if ior_ratio * ior_ratio * sin_theta_squared > 1.0 {
        return None; // 全反射が起きたので、屈折ベクトルは存在しない
    }

    let perp = ior_ratio * (incident + cos_theta * normal);
    let parallel = -(1.0 - perp.length_squared()).abs().sqrt() * normal;
    
    Some(perp + parallel)
}


// 2Dレイと円の交点を計算する関数
fn intersect_circle(ray: &Ray2D, circle: &Circle) -> Option<Hit2D> {
    let oc = ray.origin - circle.center;
    let a = ray.direction.length_squared();
    let half_b = oc.dot(ray.direction);
    let c = oc.length_squared() - circle.radius * circle.radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrtd = discriminant.sqrt();
    let mut root = (-half_b - sqrtd) / a;

    // 交点が後ろにある場合は、もう一つの解を試す
    if root < 0.001 {
        root = (-half_b + sqrtd) / a;
        if root < 0.001 {
            return None;
        }
    }
    
    let t = root;
    let point = ray.origin + t * ray.direction;
    let normal = (point - circle.center).normalize();
    
    Some(Hit2D { t, point, normal })
}

// 2Dレイと線分の交点を計算する関数
fn intersect_line(ray: &Ray2D, line: &LineSegment) -> Option<Hit2D> {
    let v1 = ray.origin - line.p1;
    let v2 = line.p2 - line.p1;
    let v3 = Vec2::new(-ray.direction.y, ray.direction.x);

    let dot_v2_v3 = v2.dot(v3);
    if dot_v2_v3.abs() < 1e-6 { // 平行な場合
        return None;
    }

    let t1 = v2.perp_dot(v1) / dot_v2_v3;
    let t2 = v1.dot(v3) / dot_v2_v3;

    if t1 >= 0.0 && (0.0..=1.0).contains(&t2) {
        let point = ray.origin + t1 * ray.direction;
        let normal = v2.perp().normalize(); // perp()で垂線ベクトルを取得
        return Some(Hit2D { t: t1, point, normal });
    }

    None
}
*/

// 2Dmain関数
/* 
use std::error::Error;
use csv::Writer;

fn main() -> Result<(), Box<dyn Error>> {
    // --- 1. シーンのセットアップ ---
    let mut scene: Vec<Box<dyn Hittable2D>> = Vec::new(); // Hittable2Dトレイトを使う

    // 大きなガラスの円（レンズ）を追加
    scene.push(Box::new(Circle {
        center: Vec2::new(20.0, 0.0),
        radius: 15.0,
        material: Material::Glass { ior: 1.5 },
    }));

    // 平面鏡を追加
    scene.push(Box::new(LineSegment {
        p1: Vec2::new(30.0, -20.0),
        p2: Vec2::new(30.0, 20.0),
        material: Material::Mirror,
    }));

    // --- 2. 初期光線の設定 ---
    let mut ray = Ray2D {
    origin: Vec2::new(-30.0, 5.0),
    direction: Vec2::new(1.0, 0.0).normalize(),
    current_ior: 1.0, // ★★★ 初期媒質は空気（屈折率1.0）
};

    // --- 3. 光路の追跡 ---
    let mut path_points: Vec<Vec2> = vec![ray.origin]; // 最初の点を記録
    let max_bounces = 10;

    // main関数内の追跡ループ部分を書き換え

for _ in 0..max_bounces {
    let mut closest_hit: Option<Hit2D> = None;
    let mut hit_material = Material::Mirror; // 仮

    // シーン内の全オブジェクトと衝突判定
    for object in &scene {
        if let Some(hit) = object.intersect(&ray) {
            if closest_hit.is_none() || hit.t < closest_hit.as_ref().unwrap().t {
                closest_hit = Some(hit);
                hit_material = object.get_material();
            }
        }
    }

    if let Some(hit) = closest_hit {
        path_points.push(hit.point);

        match hit_material {
            Material::Mirror => {
                // 反射のロジックは変更なし
                let new_direction = reflect(ray.direction, hit.normal);
                ray.origin = hit.point + new_direction * 0.001;
                ray.direction = new_direction;
            }
            Material::Glass { ior: material_ior } => {
                let outward_normal: Vec2;
                let n1: f32;
                let n2: f32;

                // 光線がオブジェクトの表面・裏面のどちらに当たったか判定
                if ray.direction.dot(hit.normal) < 0.0 {
                    // 外から中へ（表面にヒット）
                    outward_normal = hit.normal;
                    n1 = ray.current_ior;     // ★ 現在のレイの屈折率を使用
                    n2 = material_ior;        // ★ 衝突した物体の屈折率
                } else {
                    // 中から外へ（裏面にヒット）
                    outward_normal = -hit.normal; // 法線を反転
                    n1 = material_ior;        // ★ 現在（物体内）の屈折率
                    // ★★★ 次の媒質の屈折率をどう知るか？
                    // ここでは一旦、外側は常に空気(1.0)だと仮定する。
                    // より高度化するなら、衝突判定が「次にどの媒質に入るか」も返す必要がある。
                    // しかし、まずはこの仮定で進めるのが現実的。
                    n2 = 1.0; 
                }

                let ior_ratio = n1 / n2;

                // 屈折を試みる
                if let Some(refracted_dir) = refract(ray.direction, outward_normal, ior_ratio) {
                    // 屈折した場合、次のレイの屈折率を更新
                    ray.direction = refracted_dir;
                    ray.current_ior = n2;
                } else {
                    // 全反射が起きたので、代わりに反射させる
                    // この場合、媒質は変わらないので current_ior はそのまま
                    ray.direction = reflect(ray.direction, hit.normal);
                }
                ray.origin = hit.point + ray.direction * 0.001;
            }
        }
    } else {
        path_points.push(ray.origin + ray.direction * 100.0);
        break;
    }
}
*/
fn main() -> Result<(), Box<dyn Error>> {
    let mut scene: Vec<Box<dyn Hittable>> = Vec::new();

    let glass_material = Material::Glass { ior: 1.5 };

    // レンズの左側の球面
    let sphere1 = Box::new(Sphere {
        center: Vec3::new(-2.0, 0.0, 0.0), // 中心を少しずらす
        radius: 12.0,
        material: glass_material,
    });
    
    // レンズの右側の球面
    let sphere2 = Box::new(Sphere {
        center: Vec3::new(2.0, 0.0, 0.0),
        radius: 12.0,
        material: glass_material,
    });

    // 2つの球の積集合として凸レンズを定義
    let lens = Box::new(CSGObject {
        left: sphere1,
        right: sphere2,
        operation: CsgOperation::Intersection,
    });

    scene.push(lens);


    println!("CSGオブジェクトの準備ができました。");

    // --- 2. 初期光線の設定 ---
    let mut ray = Ray {
        origin: Vec3::new(-20.0, 2.0, 0.0),
        direction: Vec3::new(1.0, 0.0, 0.0).normalize(),
        current_ior: 1.0, // 空気からスタート
    };

    // --- 3. 光路の追跡 ---
    let mut path_points: Vec<Vec3> = vec![ray.origin];
    let max_bounces = 10;

    for _ in 0..max_bounces {
    let mut closest_hit_record: Option<HitRecord> = None;
    let mut t_closest = f32::INFINITY;

    for object in &scene {
        if let Some(hits) = object.intersect_all(&ray, 0.001, t_closest) {
            if let Some(first_hit) = hits.first() {
                if first_hit.t < t_closest {
                    t_closest = first_hit.t;
                    closest_hit_record = Some(*first_hit);
                }
            }
        }
    }


        if let Some(hit) = closest_hit_record {
        path_points.push(hit.point);

        // ★★★ ここが非常にシンプルになった ★★★
        let material = hit.material; // HitRecordから直接マテリアルを取得！

        match material {
            Material::Mirror => {
                ray.direction = reflect(ray.direction, hit.normal);
            }
            Material::Glass { ior: material_ior } => {
                let n1 = ray.current_ior;
                let n2 = if hit.front_face { material_ior } else { 1.0 };
                let ior_ratio = n1 / n2;

                if let Some(refracted_dir) = refract(ray.direction, hit.normal, ior_ratio) {
                    ray.direction = refracted_dir;
                    ray.current_ior = n2;
                } else {
                    ray.direction = reflect(ray.direction, hit.normal);
                }
            }
        }
        ray.origin = hit.point + ray.direction * 0.001;
    } else {
        path_points.push(ray.origin + ray.direction * 200.0);
        break;
    }
    }
    
    // --- 4. 結果をCSVファイルに出力 ---
use csv::Writer;
let mut wtr = Writer::from_path("path_3d.csv")?;
wtr.write_record(&["x", "y", "z"])?; // ヘッダー
for point in path_points {
    wtr.write_record(&[
        point.x.to_string(),
        point.y.to_string(),
        point.z.to_string(),
    ])?;
}
wtr.flush()?;

println!("3D光路を 'path_3d.csv' に出力しました。");
println!("可視化スクリプトで結果を確認してください。");

Ok(())
}

/* 
    // --- 4. 2D結果をCSVファイルに出力 ---
    let mut wtr = Writer::from_path("path_output.csv")?;
    wtr.write_record(&["x", "y"])?; // ヘッダー
    for point in path_points {
        wtr.write_record(&[point.x.to_string(), point.y.to_string()])?;
    }
    wtr.flush()?;

    println!("光路を 'path_output.csv' に出力しました。");
    println!("Python(Matplotlib)やExcelなどで可視化してみてください。");

    Ok(())

*/