use raytracing_config::simulation_config::SimulationConfig;
use raytracing_core::Scene;
use raytracing_renderer::render;
use std::error::Error;

pub fn cli() -> Result<(), Box<dyn Error>> {
    println!("設定ファイル simulation.toml を読み込んでいます...");
    let SimulationConfig {
        scene,
        simulation_settings,
    } = SimulationConfig::load_from_path("simulation.toml")?;
    let scene: Scene = scene.into();
    let results = scene.simulate_rays(simulation_settings.into());

    println!("Rendering scene with raytracer...");
    render(&scene, &results)?;
    println!("Scene rendered to output.ppm");

    Ok(())
}