use bevy::pbr::{DirectionalLight, StandardMaterial};
use bevy::prelude::*;
use bevy_flycam::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use csgrs::traits::CSG;
use rand::{self, Rng};
use raytracing_core::Scene;
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
    let scene = &render_scene.0;
    let results = &path_data.0;
    // 光の軌跡の描画
    spawn_arrows(&mut commands, &mut meshes, &mut materials, results);
    commands.spawn(DirectionalLight {
        shadows_enabled: true,
        ..default()
    });
    //commands.spawn((Camera3d::default(),));
}

fn spawn_object(
    scene: &Scene,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
) {
    let object_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    for object_def in &scene.objects {
        // 1. TOMLの定義からcsgrsのGeomオブジェクトを構築
        let csg_geom = shape_to_csgrs_geom(&object_def.shape);

        // 2. csgrsのGeomからBevyのMeshへ変換
        let bevy_mesh = csgrs_geom_to_bevy_mesh(csg_geom);

        // 3. 変換したメッシュをスポーン
        commands.spawn((
            Mesh3d(meshes.add(bevy_mesh)),
            MeshMaterial3d(object_material.clone()),
            Transform::from_xyz(
                object_def.transform.position[0],
                object_def.transform.position[1],
                object_def.transform.position[2],
            ),
        ));
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

fn shape_to_csgrs_geom(shape: &Shape) -> Geom {
    match shape {
        Shape::Sphere { radius } => Geom::new(CsgSphere::new(Vector::default(), *radius)),
        Shape::Box { size } => {
            let half_size = Vector::new(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0);
            Geom::new(Cube::new(half_size))
        }
        Shape::Cylinder { radius, height } => {
            let start = Vector::new(0.0, -height / 2.0, 0.0);
            let end = Vector::new(0.0, height / 2.0, 0.0);
            Geom::new(CsgCylinder::new(start, end, *radius))
        }
        Shape::Difference { a, b } => {
            let geom_a = shape_to_csgrs_geom(a);
            let geom_b = shape_to_csgrs_geom(b);
            geom_a.difference(&geom_b) // メソッドチェーンで演算
        }
        Shape::Union { a, b } => {
            let geom_a = shape_to_csgrs_geom(a);
            let geom_b = shape_to_csgrs_geom(b);
            geom_a.union(&geom_b)
        }
    }
}

// ✨ ヘルパー関数2: csgrsのGeomをBevyのMeshに手動で変換
fn csgrs_geom_to_bevy_mesh(geom: Geom) -> Mesh {
    // 演算を実行し、ポリゴン（三角形）のリストを取得
    let triangles = geom.to_triangles().unwrap_or_default();

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new(); // UVはダミー値を設定

    // 各三角形の頂点情報をBevyのMeshが要求する形式のVecに格納していく
    for tri in triangles {
        for vertex in tri.vertices {
            positions.push([
                vertex.pos.x as f32,
                vertex.pos.y as f32,
                vertex.pos.z as f32,
            ]);
            normals.push([
                vertex.normal.x as f32,
                vertex.normal.y as f32,
                vertex.normal.z as f32,
            ]);
            uvs.push([0.0, 0.0]); // UV座標は今回使わないのでダミー
        }
    }

    // BevyのMeshを生成
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        Default::default(),
    );
    // Meshに頂点属性を設定
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
