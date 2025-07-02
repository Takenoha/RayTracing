use std::{error::Error, path::Path};

use serde::Deserialize;

use crate::Scene;

use super::{ObjectConfig, RayConfig, SimulationSettingsConfig};

#[derive(Deserialize)]
pub struct SceneConfig {
    pub rays: Vec<RayConfig>,
    pub objects: Vec<ObjectConfig>,
}

impl Into<Scene> for SceneConfig {
    fn into(self) -> Scene {
        Scene {
            objects: self.objects.into_iter().map(Into::into).collect(),
            rays: self.rays.into_iter().map(Into::into).collect(),
        }
    }
}
