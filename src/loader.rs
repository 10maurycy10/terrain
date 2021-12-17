use bevy::prelude::*;
use std::collections::HashMap;
use bevy::render::camera::Camera;
use crate::map;
use crate::chunk::Map;

pub struct UnloadMarker;

pub struct Data {
    loader: HashMap<(i32,i32),Entity>
}

pub fn init(mut commands: Commands) {
    commands.insert_resource(Data {
        loader: HashMap::new()
    });
}

pub fn load(
    mut commands: Commands,
    cameras: Query<&GlobalTransform, With<Camera>>,
    mut data: ResMut<Data>
    
) {
    let c = cameras.iter().next().unwrap();
    
    let s = map::getchunksize();
    
    let x = (c.translation.x / s) as i32;
    let y = (c.translation.z / s) as i32;
    
    for cx in (x-4)..(x+4) {
        for cy in (y-4)..(y+4) {
            match data.loader.get(&(cx,cy)) {
                Some(_) => continue,
                None => ()
            }
            let xf = cx as f32;
            let yf = cy as f32;
            let mut e = commands.spawn();
            let spawned = e.insert(Map::new_with_transform(Transform::from_xyz(s*xf,0.0,s*yf),(xf,yf),e.id())).id();
            data.loader.insert((cx,cy),spawned);
        }
    }
    
    let mut v: Vec<(i32,i32)> = Vec::new();
    
    for (k, id) in data.loader.iter() {
         if ((x - k.0).abs()>6) || ((y - k.1).abs())>6 {
            println!("{:?} marked for unload",k);
            v.push(*k);
            commands.entity(*id).insert(UnloadMarker {});
         }
    }
    
    for i in v.iter() {
        data.loader.remove(i);
    }
    
}

pub fn unload(
    mut commands: Commands,
    maps: Query<&Map, With<UnloadMarker>>,
) {
    for i in maps.iter() {
        match i.render {
            None => (),
            Some(i) => commands.entity(i).despawn()
        }
        match i.this {
            None => (),
            Some(i) => {
                commands.entity(i).despawn();
            }
        }
    }
}
