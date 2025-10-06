use raytracing_core::SimulationSettingsConfig as CoreSimulationSettingsConfig;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
pub struct SimulationSettingsConfig {
    pub infinity_distance: f32,
    pub max_bounces: u32,
    #[serde(default = "default_samples_per_pixel")]
    pub samples_per_pixel: u32,
    #[serde(default = "default_image_width")]
    pub image_width: u32,
    #[serde(default = "default_image_height")]
    pub image_height: u32,
}

fn default_samples_per_pixel() -> u32 {
    100
}

fn default_image_width() -> u32 {
    800
}

fn default_image_height() -> u32 {
    600
}

impl From<SimulationSettingsConfig> for CoreSimulationSettingsConfig {
    fn from(config: SimulationSettingsConfig) -> CoreSimulationSettingsConfig {
        CoreSimulationSettingsConfig {
            infinity_distance: config.infinity_distance,
            max_bounces: config.max_bounces,
        }
    }
}
