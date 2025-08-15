use bevy::pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy_flycam::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use csgrs::traits::CSG;
use rand::{self, Rng};
use raytracing_core::{Hittable, InfiniteCone, Scene, RenderableShape};
type Csmesh = csgrs::mesh::Mesh<()>;
#[derive(Resource)]
pub struct RenderScene(pub Scene);
#[derive(Resource)]
pub struct PathData(pub Vec<Vec<Vec3>>);

pub fn render_core(
    scene: Scene,
    results: Vec<Vec<Vec3>>,
) -> Result<(), Box<dyn std::error::Error>> {
    render_main(scene, results);
    Ok(())
}

fn render_main(scene: Scene, results: Vec<Vec<Vec3>>) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(PlayerPlugin)
        .insert_resource(RenderScene(scene))
        .insert_resource(PathData(results))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    render_scene: Res<RenderScene>,
    path_data: Res<PathData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let results = &path_data.0;
    // 光の軌跡の描画
    spawn_arrows(&mut commands, &mut meshes, &mut materials, results);

    // シーンのオブジェクトを描画
    spawn_scene_objects(&mut commands, &mut meshes, &mut materials, &render_scene);

    commands.spawn(DirectionalLight {
        shadows_enabled: true,
        ..default()
    });
    //commands.spawn((Camera3d::default(),));
}

fn spawn_scene_objects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    render_scene: &Res<RenderScene>,
) {
    let object_material = materials.add(Color::srgb(0.8, 0.7, 0.6));

    for object in &render_scene.0.objects {
        if let Some(shape) = object.get_renderable_shape() {
            let transform = object.get_transform();
            let bevy_transform = Transform::from_matrix(transform);

            let mesh = match shape {
                RenderableShape::Sphere { radius } => meshes.add(Sphere { radius }),
                RenderableShape::Box { size } => meshes.add(Cuboid::new(size.x, size.y, size.z)),
                RenderableShape::Plane { normal } => {
                    meshes.add(Plane3d::new(normal.into(), Vec2::splat(1000.0)))
                }
                RenderableShape::Cylinder { height, radius } => {
                    meshes.add(Cylinder {
                        half_height: height / 2.0,
                        radius,
                        ..default()
                    })
                }
                RenderableShape::Cone { height, angle_deg } => {
                    let radius = height * angle_deg.to_radians().tan();
                    meshes.add(Cone {
                        height,
                        radius,
                        ..default()
                    })
                }
                _ => continue, // Ignore Wedge and Lens for now
            };

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(object_material.clone()),
                bevy_transform,
            ));
        }
    }
}


fn spawn_arrows(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    results: &Vec<Vec<Vec3>>,
) {
    let mut arrow_material = materials.add(Color::srgb(0.1, 0.1, 0.1));

    for arrows in results {
        let mut rng = rand::rng();
        let r: f32 = rng.random::<f32>();
        let g: f32 = rng.random::<f32>();
        let b: f32 = rng.random::<f32>();
        let random_color = Color::srgb(r, g, b);
        arrow_material = materials.add(random_color);
        for pair in arrows.windows(2) {
            spawn_arrow(
                commands,
                meshes,
                arrow_material.clone(),
                pair[0].into(),
                pair[1].into(),
            );
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
    let half_length = direction.length() / 2.0;
    if half_length < 0.001 {
        return;
    }

    let mid_point = start + direction / 2.0;
    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
    commands.spawn((
        Mesh3d(meshes.add(Cylinder {
            radius: 0.02,
            half_height: half_length,
        })),
        MeshMaterial3d(material.clone()),
        Transform {
            translation: mid_point,
            rotation,
            ..default()
        },
    ));

    // 矢印の先端
    commands.spawn((
        Mesh3d(meshes.add(Cone {
            radius: 0.08,
            height: 0.2,
        })),
        MeshMaterial3d(material),
        Transform {
            translation: end,
            rotation,
            ..default()
        },
    ));
}
