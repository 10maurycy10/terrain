/// ecs wraper arround map.rs

use bevy::render::mesh::Mesh;
use bevy::prelude::*;
use crate::map;

/// the componet represienting a chunk
#[derive(Component)]
pub struct Map {
    pub hightmap: Option<map::ChunkData<f32>>,
    tex: Option<Image>,
    mesh: Option<Mesh>,
    pub render: Option<Entity>,
    pub this: Option<Entity>,
//    wireframe: bool,
    transform: Transform,
    seed: (f32,f32)
}

impl Map {
    /// init a map componet, insert this into an entity to have it generated and rendered
    /// e is an enity that will be despawned when the map is unloaded.
    pub fn new_with_transform(t: Transform,seed: (f32,f32), e: Entity) -> Map {
        Map {
            hightmap: None, 
            tex: None, 
            mesh: None, 
            render: None,
            this: Some(e),
  //          wireframe: false,
            transform: t,
            seed
        }
    }
}

/// generate and load maps for all entitys with The map component 
pub fn generate_maps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut maps: Query<&mut Map>,
    mut textures: ResMut<Assets<Image>>,

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
            unlit: false,
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        });
        
        let mut w = commands.spawn(PbrBundle {
            mesh: meshes.add(m),
            material: material_handle,
            transform: map.transform,
            ..Default::default()
        });
        
//         if map.wireframe {
//             w.insert(Wireframe);
//         }
        map.render = Some(w.id());
        
        println!("map added to renderer")
    } 
}
