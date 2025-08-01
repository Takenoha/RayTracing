use bevy_render_cli::render_cli;
use csv::Writer;
use raytracing_config::simulation_config::SimulationConfig;
use raytracing_core::Scene;
use std::error::Error;

pub fn cli() -> Result<(), Box<dyn Error>> {
    println!("設定ファイル simulation.toml を読み込んでいます...");
    let SimulationConfig {
        scene,
        simulation_settings,
    } = SimulationConfig::load_from_path("simulation.toml")?;
    let scene: Scene = scene.into();
    let results = scene.simulate_rays(simulation_settings.into());
    render_cli(scene, results.clone());
    // --- 3c. 結果を光路ごとに別々のCSVファイルに出力 ---
    for (i, result) in results.into_iter().enumerate() {
        let file_name = format!("./dist/path_{}.csv", i);
        let mut wtr = Writer::from_path(&file_name)?;
        wtr.write_record(&["x", "y", "z"])?;
        for point in result {
            wtr.write_record(&[
                point.x.to_string(),
                point.y.to_string(),
                point.z.to_string(),
            ])?;
        }
        wtr.flush()?;
        println!("光路 {} を '{}' に出力しました。", i, file_name);
    }

    Ok(())
}
