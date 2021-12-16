use bevy::prelude::*;

use bevy::{
    render::wireframe::{WireframePlugin},
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
    asset::Asset
};
extern crate nalgebra as na;
use bevy::input::keyboard::KeyboardInput;

mod map;
mod input;
mod chunk;

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
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(input::set_up.system())
        .add_startup_system(setup.system())
        .add_startup_system(chunk::insert_map.system())
        .add_system(chunk::generate_maps.system())
        .add_system(input::keyboard_events.system())
        .run();
}

/// start loading assets and add boilerplate entitys
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // load assets
    let a: Handle<Texture> = asset_server.load(map::ASSETS_GRASS);
    let b: Handle<Texture> = asset_server.load(map::ASSETS_WATER);
    let c: Handle<Texture> = asset_server.load(map::ASSETS_SAND);
    
    // set up the camera
    let mut camera = PerspectiveCameraBundle::new_3d();
    //camera.orthographic_projection.scale = 3.0;
    camera.transform = Transform::from_xyz(-2.0, 5.0, -2.0).looking_at(Vec3::new(3.0,0.0,3.0), Vec3::Y);
     
    // camera
    commands.spawn_bundle(camera);
    
    // dirty hack to prevent asset unloading
    commands.spawn().insert(a).insert(b).insert(c);
    
     // light
     commands.spawn_bundle(LightBundle {
         transform: Transform::from_xyz(-3.0, 6.0, -3.0),
         ..Default::default()
     });
}
