use std::{error::Error, path::Path};

use serde::Deserialize;

use super::{ObjectConfig, RayConfig, SimulationSettingsConfig};

#[derive(Deserialize)]
pub struct SceneConfig {
    pub rays: Vec<RayConfig>,
    pub objects: Vec<ObjectConfig>,
}
