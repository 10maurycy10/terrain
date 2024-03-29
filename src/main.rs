use bevy::prelude::*;

use bevy::{
    pbr::wireframe::WireframePlugin,
//    render::options::{WgpuFeatures, WgpuOptions},
};
extern crate nalgebra as na;

/// liner interpolation ac is the coificent for a
pub fn lerp(a: f32,b: f32, ac: f32) -> f32 {
    a*ac + b*(1.0-ac)
}

mod map;
mod input;
mod chunk;
mod loader;
mod reg;

fn main() {
    App::new()
//        .insert_resource(Window {
//            title: "Terrain".to_string(),
//            vsync: false,
//            ..Default::default()
//        })
//         .insert_resource(WgpuOptions {
//             features: WgpuFeatures {
//                 // The Wireframe requires NonFillPolygonMode feature
//                 features: vec![WgpuFeature::NonFillPolygonMode],
//             },
//             ..Default::default()
//         })
//        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(input::set_up)
        .add_system(input::keyboard_events)
        .add_startup_system(loader::init)
        .add_system(loader::load)
        .add_system(loader::unload)
        .add_startup_system(setup)
        .add_system(chunk::generate_maps)
        .run();
}

/// start loading assets and add boilerplate entitys
/// FIXME this leeks memory
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
    let mut camera = Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 5.0, -2.0).looking_at(Vec3::new(3.0,0.0,3.0), Vec3::Y),
        ..default()
    };
     
    // camera
    commands.spawn(camera);
    
    // Oh, no! memory leek.
    use std::mem;
    mem::forget(a);
    mem::forget(b);
    mem::forget(c);
    mem::forget(d);   
    mem::forget(e);
    
      // light
      commands.spawn(DirectionalLightBundle {
         directional_light: DirectionalLight {
             illuminance: 30000.0,
             shadows_enabled: true,
             ..Default::default()
         },
         transform: Transform::from_xyz(-5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
          ..Default::default()
      });
    
}
