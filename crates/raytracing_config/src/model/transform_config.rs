use serde::Deserialize;

fn default_position() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct TransformConfig {
    #[serde(default = "default_position")]
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation_x_deg: f32,
    #[serde(default)]
    pub rotation_y_deg: f32,
    #[serde(default)]
    pub rotation_z_deg: f32,
}
