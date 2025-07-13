use raytracing_core::SimulationSettingsConfig as CoreSimulationSettingsConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SimulationSettingsConfig {
    pub infinity_distance: f32,
    pub max_bounces: u32,
}

impl Into<CoreSimulationSettingsConfig> for SimulationSettingsConfig {
    fn into(self) -> CoreSimulationSettingsConfig {
        CoreSimulationSettingsConfig {
            infinity_distance: self.infinity_distance,
            max_bounces: self.max_bounces,
        }
    }
}
