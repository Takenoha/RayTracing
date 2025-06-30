use serde::Deserialize;

#[derive(Deserialize)]
pub struct SimulationSettingsConfig {
    pub infinity_distance: f32,
    pub max_bounces: u32,
}
