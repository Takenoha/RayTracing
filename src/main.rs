use std::error::Error;

// 3Dベクトルを扱うためにglamクレートのVec3をインポート
use glam::{Vec3};

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
    t: f32,          // レイの始点から衝突点までの距離
    point: Vec3,     // 衝突点の3D座標
    normal: Vec3,    // 衝突点における法線ベクトル
    front_face: bool,
}

trait Hittable: Send + Sync { // Send + Sync は並列処理のためのマーカー（今は気にしなくてOK）
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn get_material(&self) -> Material;
}

#[derive(Debug, Clone, Copy)]
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
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 { return None; }
        
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let point = ray.origin + t * ray.direction;
        let outward_normal = (point - self.center) / self.radius;
        
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face { outward_normal } else { -outward_normal };

        Some(HitRecord { t, point, normal, front_face })
    }

    fn get_material(&self) -> Material { self.material }
}

// 無限平面
#[derive(Debug, Clone, Copy)]
struct Plane {
    point: Vec3, // 平面上の任意の点
    normal: Vec3, // 平面の法線
    material: Material,
}

impl Hittable for Plane {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() < 1e-6 { return None; } // レイは平面と平行

        let t = (self.point - ray.origin).dot(self.normal) / denom;

        if t < t_min || t_max < t { return None; }

        let point = ray.origin + t * ray.direction;
        
        let front_face = ray.direction.dot(self.normal) < 0.0;
        let normal = if front_face { self.normal } else { -self.normal };
        
        Some(HitRecord { t, point, normal, front_face })
    }

    fn get_material(&self) -> Material { self.material }
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
    // --- 1. シーンのセットアップ ---
    let mut scene: Vec<Box<dyn Hittable>> = Vec::new();

    // ガラスの球を追加
    scene.push(Box::new(Sphere {
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 10.0,
        material: Material::Glass { ior: 1.5 },
    }));

    // 反射する床を追加
    scene.push(Box::new(Plane {
        point: Vec3::new(0.0, -15.0, 0.0),
        normal: Vec3::Y, // Y軸の正方向を向いた法線
        material: Material::Mirror,
    }));

    // --- 2. 初期光線の設定 ---
    let mut ray = Ray {
        origin: Vec3::new(-20.0, 5.0, 0.0),
        direction: Vec3::new(1.0, -0.1, 0.0).normalize(),
        current_ior: 1.0, // 空気からスタート
    };

    // --- 3. 光路の追跡 ---
    let mut path_points: Vec<Vec3> = vec![ray.origin];
    let max_bounces = 10;

    for _ in 0..max_bounces {
        let mut closest_hit: Option<HitRecord> = None;
        let mut hit_material = Material::Mirror; // 仮
        let mut t_closest = f32::INFINITY;

        for object in &scene {
            if let Some(hit) = object.intersect(&ray, 0.001, t_closest) {
                t_closest = hit.t;
                closest_hit = Some(hit);
                hit_material = object.get_material();
            }
        }

        if let Some(hit) = closest_hit {
            path_points.push(hit.point);

            match hit_material {
                Material::Mirror => {
                    ray.direction = reflect(ray.direction, hit.normal);
                }
                Material::Glass { ior: material_ior } => {
                    let n1 = ray.current_ior;
                    let n2 = if hit.front_face { material_ior } else { 1.0 }; // 出る先は空気と仮定
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
    
    // --- 4. 結果をコンソールに出力 ---
    println!("光路計算が完了しました。座標点:");
    for (i, point) in path_points.iter().enumerate() {
        println!("  {}: ({:.3}, {:.3}, {:.3})", i, point.x, point.y, point.z);
    }

    Ok(())
}
/* 
    // --- 4. 結果をCSVファイルに出力 ---
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