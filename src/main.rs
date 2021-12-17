use bevy::prelude::*;

use bevy::{
    pbr::wireframe::WireframePlugin,
    render::options::{WgpuFeatures, WgpuOptions},
};
extern crate nalgebra as na;

mod map;
mod input;
mod chunk;
mod loader;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Terrain".to_string(),
            vsync: false,
            ..Default::default()
        })
//         .insert_resource(WgpuOptions {
//             features: WgpuFeatures {
//                 // The Wireframe requires NonFillPolygonMode feature
//                 features: vec![WgpuFeature::NonFillPolygonMode],
//             },
//             ..Default::default()
//         })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(input::set_up.system())
        .add_startup_system(loader::init.system())
        .add_startup_system(setup.system())
        //.add_startup_system(chunk::insert_map.system())
        .add_system(loader::load.system().before("unloader"))
        .add_system(loader::unload.system().label("unloader"))
        .add_system(chunk::generate_maps.system().before("unloader"))
        .add_system(input::keyboard_events.system().before("unloader"))
        .run();
}

/// start loading assets and add boilerplate entitys
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // load assets
    let a: Handle<Image> = asset_server.load(map::ASSETS_GRASS);
    let b: Handle<Image> = asset_server.load(map::ASSETS_WATER);
    let c: Handle<Image> = asset_server.load(map::ASSETS_SAND);
    let d: Handle<Image> = asset_server.load(map::ASSETS_SNOW);
    let e: Handle<Image> = asset_server.load(map::ASSETS_STONE);
    
    
    // set up the camera
    let mut camera = PerspectiveCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(-2.0, 5.0, -2.0).looking_at(Vec3::new(3.0,0.0,3.0), Vec3::Y);
     
    // camera
    commands.spawn_bundle(camera);
    
    // Oh, no! memory leek.
    use std::mem;
    mem::forget(a);
    mem::forget(b);
    mem::forget(c);
    mem::forget(d);   
    mem::forget(e);
    
      // light
      commands.spawn_bundle(DirectionalLightBundle {
         directional_light: DirectionalLight {
             illuminance: 30000.0,
             ..Default::default()
         },
         transform: Transform::from_xyz(-5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
          ..Default::default()
      });
}
