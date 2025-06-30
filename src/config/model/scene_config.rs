use std::{error::Error, path::Path};

use serde::Deserialize;

use super::{ObjectConfig, RayConfig, SimulationSettingsConfig};

#[derive(Deserialize)]
pub struct SceneConfig {
    pub simulation_settings: SimulationSettingsConfig,
    pub rays: Vec<RayConfig>,
    pub objects: Vec<ObjectConfig>,
}

impl SceneConfig {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<SceneConfig, Box<dyn Error>> {
        let toml_str = std::fs::read_to_string(path)?;
        let scene_config = toml::from_str(&toml_str)?;

        Ok(scene_config)
    }
}
