use bevy::prelude::*;

use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::{
    render::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use bevy::render::texture::Extent3d;
use bevy::render::texture::TextureDimension;
use bevy::render::texture::TextureFormat;

extern crate nalgebra as na;
use na::{Vector3, Rotation3};

mod map;

enum AppState {
    Loading,
    Running
}


fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Terrain".to_string(),
            vsync: true,
            ..Default::default()
        })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                // The Wireframe requires NonFillPolygonMode feature
                features: vec![WgpuFeature::NonFillPolygonMode],
            },
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(AppState::Loading)
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup.system())
        .add_system(setup_stage2.system())
        .run();
}

/// start loading assets
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // load grass
    let texture_handle: Handle<Texture> = asset_server.load("grass64.png");
}

/// set up.
fn setup_stage2(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AppState>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // if the end of the function was reached, return 
    match *state {
        AppState::Running => return,
        AppState::Loading => ()
    }
    
    // get a handle to grass64.png
    let grass_handle: Handle<Texture>  = asset_server.load("grass64.png");
    
    // if not loaded, return
    match textures.get(grass_handle.clone()) {
        Some(_) => (),
        None => return,
    }
    
    // generate chunk hightmap
    let (tex,mesh) = map::gen(&mut textures).unwrap();

    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 3.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    
    // camera
    commands.spawn_bundle(camera);

    let tex_handle = textures.add(tex);
    
    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(tex_handle.clone()),
        unlit: false,
        roughness: 0.9,
        metallic: 0.0,
        ..Default::default()
    });
    
    // world
    let mut w = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: material_handle,
        ..Default::default()
    });
    
    //w.insert(Wireframe);
    
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-3.0, 6.0, -3.0),
        ..Default::default()
    });
    
    *state = AppState::Running;
}

