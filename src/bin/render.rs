use raytracing_config::simulation_config::SimulationConfig;
use raytracing_core::Scene;
use raytracing_renderer::render;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("設定ファイル simulation.toml を読み込んでいます...");
    let config = SimulationConfig::load_from_path("simulation.toml")?;
    let scene: Scene = config.scene.into();

    println!("レンダリングを開始します...");
    let settings = config.simulation_settings;
    render(
        &scene,
        &config.camera,
        settings.image_width,
        settings.image_height,
        settings.samples_per_pixel,
        settings.max_bounces,
    );

    Ok(())
}
