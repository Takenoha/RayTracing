use std::{error::Error, path::Path};

use serde::Deserialize;

use crate::{scene_config::SceneConfig, simulation_settings_config::SimulationSettingsConfig};

#[derive(Deserialize)]
pub struct SimulationConfig {
    pub simulation_settings: SimulationSettingsConfig,
    pub scene: SceneConfig,
}

impl SimulationConfig {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<SimulationConfig, Box<dyn Error>> {
        let toml_str = std::fs::read_to_string(path)?;
        let simulation_config = toml::from_str(&toml_str)?;

        Ok(simulation_config)
    }
}
