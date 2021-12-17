/// ecs wraper arround map.rs

use bevy::render::mesh::Mesh;
use bevy::prelude::*;
use crate::map;
use bevy::render::wireframe::Wireframe;

/// the componet represienting a chunk
pub struct Map {
    pub hightmap: Option<map::ChunkData<f32>>,
    tex: Option<Texture>,
    mesh: Option<Mesh>,
    pub render: Option<Entity>,
    pub this: Option<Entity>,
    wireframe: bool,
    transform: Transform,
    seed: (f32,f32)
}

impl Map {
//     /// init a new ungenerated map
//     pub fn new(seed: (f32,f32), e: Entity) -> Map {
//         Map {
//             hightmap: None, 
//             tex: None, 
//             mesh: None, 
//             render: None,
//             this: Some(e),
//             wireframe: false,
//             transform: Transform::identity(),
//             seed
//         }
//     }
    /// init a map with a trasform
    pub fn new_with_transform(t: Transform,seed: (f32,f32), e: Entity) -> Map {
        Map {
            hightmap: None, 
            tex: None, 
            mesh: None, 
            render: None,
            this: Some(e),
            wireframe: false,
            transform: t,
            seed
        }
    }
}

/// set up.
pub fn generate_maps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut maps: Query<&mut Map>,
    mut textures: ResMut<Assets<Texture>>,

) {
    for mut map in maps.iter_mut() {
        match map.hightmap {
            Some(_) => continue,
            None => ()
        };
        let (t, m, h) = match map::gen(&mut textures,map.seed) {
            Ok(x) => x,
            Err(e) => {
                println!("{}",e);
                continue;
            },
        };
        map.hightmap = Some(h);
        map.mesh = Some(m.clone());
        map.tex = Some(t.clone());
        println!("map generated!");
        
        let tex_handle = textures.add(t);
     
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(tex_handle.clone()),
            unlit: true,
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        });
        
        let mut w = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(m),
            material: material_handle,
            transform: map.transform,
            ..Default::default()
        });
        
        if map.wireframe {
            w.insert(Wireframe);
        }
        map.render = Some(w.id());
        
        println!("map added to renderer")
    } 
}
