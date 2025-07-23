use bevy::prelude::*;
use rand::prelude::*;
use raytracing_core::Scene;

pub fn render_core(
    scene: Scene,
    results: Vec<Vec<Vec3>>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn render_main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(scene: Scene, results: Vec<Vec<Vec3>>) {}

fn spawn_arrows(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    results: Vec<Vec<Vec3>>,
) {
    let arrow_material = materials.add(Color::CYAN);
    let mut rng = rand::thread_rng();
    for arrows in results {
        let r: f32 = rng.random::<f32>();
        let g: f32 = rng.random::<f32>();
        let b: f32 = rng.random::<f32>();
        let random_color = Color::srgb(r, g, b);
        arrow_material = materials.add(random_color);
        for pair in arrows.windows(2) {
            let start_point = pair[0];
            let end_point = pair[1];
        }
    }
}

fn spawn_arrow(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    start: Vec3,
    end: Vec3,
) {
    let direction = end - start;
    let length = direction.length();
    if length < 0.001 {
        return;
    }

    let mid_point = start + direction / 2.0;

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cylinder::new(0.02, length)),
        material: material.clone(),
        transform: Transform::from_translation(mid_point)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, direction.normalize())),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cone::new(0.08, 0.2)),
        material: material,
        transform: Transform::from_translation(end)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, direction.normalize())),
        ..default()
    });
}
