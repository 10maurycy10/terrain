use noise::{NoiseFn, Perlin};
use noise::Seedable;
use nalgebra::Vector3;
use bevy::render::mesh::Mesh;
use bevy::render::texture::Extent3d;
use bevy::render::texture::TextureDimension;
use bevy::render::texture::TextureFormat;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;

/// the filename of the asset for grass
pub const ASSETS_GRASS: &str = "grass64.png";
/// the side length (px) of assets
pub const ASSET_SIZE: usize = 64;
/// the desired resolution of the map (point = point on hightmap)
pub const PIXELS_PER_POINT: usize = 16;
/// the side length of a chunk
pub const CHUNK_SIZE: usize = 32;

pub const PIXELS_PER_CHUNK: usize = PIXELS_PER_POINT*CHUNK_SIZE;
/// the aria (samples) of a chunk
pub const CHUNK_SQSIZE: usize = CHUNK_SIZE*CHUNK_SIZE;
/// size of sample on map`
pub const VOXEL_SCALE: f32 = 0.2;

/// a row major array for hightmap data
// x + z*CHUNK_SIZE
pub type ChunkData<N> = Box<[N; CHUNK_SQSIZE]>;


/// create a hightmap
pub fn genchunk() -> ChunkData<f32> {
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

/// convert a hightmap into a texture
pub fn chunktotexture(data:&ChunkData<f32>, grass : &Texture) -> Texture {
    let grass = grass.convert(TextureFormat::Rgba8UnormSrgb).unwrap();
    let tex = Texture::new_fill(
        Extent3d::new((PIXELS_PER_CHUNK) as u32,(PIXELS_PER_CHUNK) as u32,1)
        ,TextureDimension::D2,
        &(0..(PIXELS_PER_CHUNK*PIXELS_PER_CHUNK))
            .flat_map(|i| {
                let x = i%PIXELS_PER_CHUNK;
                let y = i/PIXELS_PER_CHUNK;
                //let map_x = x/PIXELS_PER_POINT;
                //let map_y = y/PIXELS_PER_POINT;
                let gidx = ((x%ASSET_SIZE)+(y%ASSET_SIZE)*ASSET_SIZE)*4;
                [grass.data[gidx + 0],grass.data[gidx + 1],grass.data[gidx + 2],255]
                //[255 as u8,255 as u8,0,255]
            })
            .collect::<Vec<u8>>()
        ,TextureFormat::Rgba8UnormSrgb
    );
    tex
}

/// convert a hightmap to a mesh
pub fn chunktomesh(hightmap: &ChunkData<f32>) -> Mesh {
    let mut position = Vec::new();
    
    // trival hightmap to vertex code. (Row major)
    for (i,hight) in hightmap.iter().enumerate() {
        let x = (i % CHUNK_SIZE) as f32;
        let z = (i / CHUNK_SIZE) as f32;
        position.push([x*VOXEL_SCALE,hight*VOXEL_SCALE,z*VOXEL_SCALE])
    }
    
    // uvs control how a texture is maped onto the mesh.
    // this will strech the entire texture over the mesh, resulting in distortion on slopes
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    for i in 0..(CHUNK_SQSIZE) {
        let x = (i % CHUNK_SIZE);
        let y = (i / CHUNK_SIZE);
        uvs.push([x as f32/CHUNK_SIZE as f32,y as f32/CHUNK_SIZE as f32]);
    }
    
    // create triangles
    let mut indeces: Vec<u32> = Vec::new();
    // for all hightmap points ...
    for i in 0..(CHUNK_SQSIZE) {
        let x = (i % CHUNK_SIZE) as u32;
        let z = (i / CHUNK_SIZE) as u32;
        // if not on +x or +y edge...
        if (x != (CHUNK_SIZE as u32-1)) && (z != (CHUNK_SIZE as u32-1)) {
            // the index of the the point next on x, y, and both.
            let nx_idx = (x+1) + z * CHUNK_SIZE as u32;
            let ny_idx = x + (z+1) * CHUNK_SIZE as u32;
            let nxy_idx = (x+1) + (z+1) * CHUNK_SIZE as u32;
            // add the first tri
            indeces.push(nx_idx);
            indeces.push(i as u32);
            indeces.push(ny_idx);
            // add a second tru
            indeces.push(nx_idx);
            indeces.push(ny_idx);
            indeces.push(nxy_idx);
        }
    }
    
    // create an array of null normals
    let mut normals = Vec::new();
    for _ in 0..(CHUNK_SQSIZE)  {
        normals.push([0., 0., 0.]);
    }
    
    // for all tris...
    for i in 0..(indeces.len()/3) {
        let idx = i*3;
        
        // get indeces of all points in tri
        let idx0 = indeces[idx+0] as usize;
        let idx1 = indeces[idx+1] as usize;
        let idx2 = indeces[idx+2] as usize;
        
        // convert to the points nalgebra vectors
        let a = Vector3::new(position[idx0][0],position[idx0][1],position[idx0][2]);
        let b = Vector3::new(position[idx1][0],position[idx1][1],position[idx1][2]);
        let c = Vector3::new(position[idx2][0],position[idx2][1],position[idx2][2]);
        
        // calculate face normals
        let v1 = b - a;
        let v2 = c - a;
        
        let n = v1.cross(&v2).normalize();
        
        // add the face normal to the normals for all points ajacent to tri.
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
    
    // for all normals...
    for i in 0..(normals.len()) {
        // normalize normal.
        let m= Vector3::from_row_slice(&normals[i]).normalize();
        let n = m.column(0);
        normals[i] = [n[0], n[1], n[2]];
    }
    
    // pack all data into bevy mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, position);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indeces)));
    mesh
}

/// helper function to generate textures and mesh, fails if assets are not loaded.
pub fn gen(assets: &mut Assets<Texture>) -> Result<(Texture,Mesh),String> {
    let hightmap = genchunk();
    let mesh = chunktomesh(&hightmap);
    let tex = chunktotexture(
        &hightmap,
        assets.get(ASSETS_GRASS)
            .map_or_else(|| Err("cant get grass.".to_string()), |x| Ok(x))?
    );
    Ok((tex,mesh))
}
