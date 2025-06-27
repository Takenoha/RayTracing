// 他のHittableオブジェクトに変換を適用するためのラッパー
struct Transform {
    pub object: Box<dyn Hittable>,
    pub transform: Mat4,         // ローカル空間 -> ワールド空間への変換
    pub inverse_transform: Mat4, // ワールド空間 -> ローカル空間への変換
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