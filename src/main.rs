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

use noise::{NoiseFn, Perlin};
use noise::Seedable;

const CHUNK_SIZE: usize = 32;
const CHUNK_SQSIZE: usize = CHUNK_SIZE*CHUNK_SIZE;
const VOXEL_SCALE: f32 = 0.2;

enum AppState {
    Loading,
    Running
}

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
        
        let f1 = perlin.get([x as f64/3.0,y as f64/3.0]) as f32 * 0.2;
        let f2 = perlin.get([x as f64/10.0,y as f64/10.0]) as f32 * 2.0;
        
        *ptr = f1 + f2;
        
        //*ptr = 0.0;
    }
    return Box::new(cdata)
}

fn chunktotexture(data:&ChunkData<f32>, grass : &Texture) -> Texture {
    let grass = grass.convert(TextureFormat::Rgba8UnormSrgb).unwrap();
    let tex = Texture::new_fill(
        Extent3d::new(16*CHUNK_SIZE as u32,16*CHUNK_SIZE as u32,1)
        ,TextureDimension::D2,
        &(0..(CHUNK_SQSIZE*16*16))
            .flat_map(|i| {
                let x = i%(CHUNK_SIZE*16);
                let y = i/(CHUNK_SIZE*16);
                let gidx = ((x%64)+(y%64)*64)*4;
                [grass.data[gidx + 0],grass.data[gidx + 1],grass.data[gidx + 2],255]
                //[255 as u8,255 as u8,0,255]
            })
            .collect::<Vec<u8>>()
        ,TextureFormat::Rgba8UnormSrgb
    );
    tex
}

fn chunktomesh(hightmap: &ChunkData<f32>) -> Mesh {
    let mut position = Vec::new();
    for (i,hight) in hightmap.iter().enumerate() {
        let x = (i % CHUNK_SIZE) as f32;
        let z = (i / CHUNK_SIZE) as f32;
        position.push([x*VOXEL_SCALE,hight*VOXEL_SCALE,z*VOXEL_SCALE])
    }
    
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    for i in 0..(CHUNK_SQSIZE) {
        let x = (i % CHUNK_SIZE);
        let y = (i / CHUNK_SIZE);
        uvs.push([x as f32/CHUNK_SIZE as f32,y as f32/CHUNK_SIZE as f32]);
    }
    
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
    let chunk = genchunk();
    
    // generate texture from hightmap
    let tex = chunktotexture(&chunk,textures.get(grass_handle).unwrap());
    
    // generate mesh from hightmap
    let mesh = chunktomesh(&chunk);

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

