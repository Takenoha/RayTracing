use bevy::prelude::*;
use bevy_render_core::render_core;
use raytracing_core::Scene;
pub fn render_cli(scene: Scene, results: Vec<Vec<Vec3>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("レンダー起動");

    render_core(scene, results);
    Ok(())
}
