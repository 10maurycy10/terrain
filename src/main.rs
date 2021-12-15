use bevy::prelude::*;

use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::{
    render::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};

extern crate nalgebra as na;
use na::{Vector3, Rotation3};

use noise::{NoiseFn, Perlin};
use noise::Seedable;

const CHUNK_SIZE: usize = 16;
const CHUNK_SQSIZE: usize = 16*16;
const VOXEL_SCALE: f32 = 0.2;

// x + z*CHUNK_SIZE
type ChunkData<N> = Box<[N; CHUNK_SQSIZE]>;

// function to create a hightmap
fn genchunk() -> ChunkData<f32> {
    // todo, use mabey uninit
    let mut cdata = [0.0_f32; CHUNK_SQSIZE];
    let perlin = Perlin::new();
    perlin.set_seed(10);
    
    for (idx, ptr) in cdata.iter_mut().enumerate() {
        let x = (idx % CHUNK_SIZE) as f32;
        let y = (idx / CHUNK_SIZE) as f32;
        *ptr =  perlin.get([x as f64/5.,y as f64/5.]) as f32;
    }
    return Box::new(cdata)
}

fn chunktomesh(hightmap: &ChunkData<f32>) -> Mesh {
    let mut position = Vec::new();
    for (i,hight) in hightmap.iter().enumerate() {
        let x = (i % CHUNK_SIZE) as f32;
        let z = (i / CHUNK_SIZE) as f32;
        position.push([x*VOXEL_SCALE,hight*VOXEL_SCALE,z*VOXEL_SCALE])
    }
    
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    for _ in 0..(CHUNK_SQSIZE) {
        uvs.push([0., 0.]);
    }
    
    // TODO FIX THIS !!
    let mut normals = Vec::new();
    for _ in 0..(CHUNK_SQSIZE)  {
        normals.push([0., 0., 0.]);
    }
    
    let mut indeces: Vec<u32> = Vec::new();
    for i in 0..(CHUNK_SQSIZE) {
        let x = (i % CHUNK_SIZE) as u32;
        let z = (i / CHUNK_SIZE) as u32;
        if (x != (CHUNK_SIZE as u32-1)) && (z != (CHUNK_SIZE as u32-1)) {
            let nx_idx = (x+1) + z * CHUNK_SIZE as u32;
            let ny_idx = x + (z+1) * CHUNK_SIZE as u32;
            let nxy_idx = (x+1) + (z+1) * CHUNK_SIZE as u32;
            //println!("{} {} {}",i,nx_idx,ny_idx);
            indeces.push(nx_idx);
            indeces.push(i as u32);
            indeces.push(ny_idx);
            
            indeces.push(nx_idx);
            indeces.push(ny_idx);
            indeces.push(nxy_idx);
        } else {
            //println!("REJECTING: {} {}",x,z)
        }
    }
    
    for i in 0..(indeces.len()/3) {
        let idx = i*3;
        
        let idx0 = indeces[idx+0] as usize;
        let idx1 = indeces[idx+1] as usize;
        let idx2 = indeces[idx+2] as usize;
        
        let a = Vector3::new(position[idx0][0],position[idx0][1],position[idx0][2]);
        let b = Vector3::new(position[idx1][0],position[idx1][1],position[idx1][2]);
        let c = Vector3::new(position[idx2][0],position[idx2][1],position[idx2][2]);
        
        let v1 = b - a;
        let v2 = c - a;
        
        let n = v1.cross(&v2).normalize();
        
        let m0 = (n + Vector3::from_row_slice(&normals[idx0]));
        let m1 = (n + Vector3::from_row_slice(&normals[idx1]));
        let m2 = (n + Vector3::from_row_slice(&normals[idx2]));
        
        let n0 = m0.column(0);
        let n1 = m1.column(0);
        let n2 = m2.column(0);
        
        normals[idx0] = [n0[0], n0[1], n0[2]];
        normals[idx1] = [n1[0], n1[1], n1[2]];
        normals[idx2] = [n2[0], n2[1], n2[2]];
    }
    
    for i in 0..(normals.len()) {
        let m= Vector3::from_row_slice(&normals[i]).normalize();
        let n = m.column(0);
        normals[i] = [n[0], n[1], n[2]];
    }
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, position);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indeces)));
    mesh
}

fn create_triangle() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    let mut normals = Vec::new();
    for _ in 0..3 {
        normals.push([0., 1., 0.]);
    }
    
    let mut uvs: Vec<[f32; 2]> = vec![];
    for _ in 0..3 {
        uvs.push([0., 0.]);
    }
    
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vec![[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0]]);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(vec![0,1,2])));
    mesh
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
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 3.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    
    let chunk = genchunk();
    
    let mesh = chunktomesh(&chunk);

    // camera
    commands.spawn_bundle(camera);

    // world
    let mut w = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    
//    w.insert(Wireframe);
    
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(-3.0, 6.0, -3.0),
        ..Default::default()
    });
}
