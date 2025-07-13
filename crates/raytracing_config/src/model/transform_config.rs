use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct TransformConfig {
    pub position: [f32; 3],
    pub rotation_y_deg: f32,
}
