use noise::{NoiseFn, Perlin};
use noise::Seedable;
use nalgebra::Vector3;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_resource::TextureFormat;
use bevy::render::texture::Image;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use crate::reg;
use crate::lerp;

/// the filenames of the assets
pub const ASSETS_GRASS: &str = "grass16.png";
pub const ASSETS_WATER: &str = "water16.png";
pub const ASSETS_SAND: &str = "sand16.png";
pub const ASSETS_SNOW: &str = "snow16.png";
pub const ASSETS_STONE: &str = "stone16.png";
/// the side length (px) of assets
pub const ASSET_SIZE: usize = 16;
/// the desired resolution of the map (point = point on hightmap)
pub const PIXELS_PER_POINT: usize = 4;
/// the side length of a chunk
pub const CHUNK_SIZE: usize = 64;

pub const PIXELS_PER_CHUNK: usize = PIXELS_PER_POINT*CHUNK_SIZE;
/// the aria (samples) of a chunk
pub const CHUNK_SQSIZE: usize = CHUNK_SIZE*CHUNK_SIZE;
/// size of sample on map`
pub const VOXEL_SCALE: f32 = 0.4;

/// a row major array for hightmap data
// x + z*CHUNK_SIZE
pub type ChunkData<N> = Box<[N; CHUNK_SQSIZE]>;

/// calculate the size of the chunk mesh
pub fn getchunksize() -> f32 {
    VOXEL_SCALE * (CHUNK_SIZE-1) as f32
}

/// create a regdata for a chunk.
pub fn genchunkreg(seed: (f32,f32)) -> reg::Regdata {
    let regseed = (seed.0 * CHUNK_SIZE as f32,seed.1 * CHUNK_SIZE as f32);
    reg::newreg(regseed)
}

fn ravien(h: f32) -> f32 {
    if h > 5.0 && h < 7.0 {
            h - 6.7 
    } else {
        h
    }
}

fn clifs(h: f32) -> f32 {
    if h > 10.0 {
            h + 4.0 
    } else {
        h
    }
}

fn fiords(h: f32) -> f32 {
    if h < 10.0 {
            h - 9.0 
    } else {
        h
    }
}

/// create a hightmap
/// regs is a row major array containg a 2x2 grid regions, with 0,0 being the chunk.
/// the additional regons are used to blend generation
pub fn genchunk(seed: (f32,f32),regs: &[reg::Regdata;4]) -> ChunkData<f32> {
    // todo, use mabey uninit
    let mut cdata = [0.0_f32; CHUNK_SQSIZE];
    let perlin = Perlin::new();
    perlin.set_seed(100);
    
    let ox = seed.0*(CHUNK_SIZE-1) as f32;
    let oy = seed.1*(CHUNK_SIZE-1) as f32;
    
    for (idx, ptr) in cdata.iter_mut().enumerate() {
        let x = (idx % CHUNK_SIZE) as f32;
        let y = (idx / CHUNK_SIZE) as f32;
        
        // x and y on a scale from 0.0 to 1.0
        let nx = x as f32 / CHUNK_SIZE as f32;
        let ny = y as f32 / CHUNK_SIZE as f32;
        
        let wx = ox + x;
        let wy = oy + y;
        
        let f1 = perlin.get([wx as f64/2.0,wy as f64/2.0]) as f32 * 0.4;
        let f2 = perlin.get([wx as f64/20.0,wy as f64/20.0]) as f32 * 2.0;
        let f3 = perlin.get([wx as f64/200.0,wy as f64/200.0]) as f32 * 30.0;
        
        *ptr = f1 + f2 + f3;
        
        let local_rev = lerp(
            lerp(regs[3].raviens,regs[2].raviens,nx),
            lerp(regs[1].raviens,regs[0].raviens,nx),
            ny
        );
        
        *ptr = lerp(ravien(*ptr),*ptr,local_rev);
       
        let local_clifs = lerp(
            lerp(regs[3].clifs,regs[2].clifs,nx),
            lerp(regs[1].clifs,regs[0].clifs,nx),
            ny
        );
       
        *ptr = lerp(clifs(*ptr),*ptr,local_clifs);
        
        let local_fiords = lerp(
            lerp(regs[3].fiords,regs[2].fiords,nx),
            lerp(regs[1].fiords,regs[0].fiords,nx),
            ny
        );
        
        *ptr = lerp(fiords(*ptr),*ptr,local_fiords);
       
        if *ptr < -0.71 {
            *ptr = -0.71
        }
        
        //*ptr = 0.0;
    }
    return Box::new(cdata)
}

/// create a slopemap from hightmap
pub fn genslope(data: &ChunkData<f32>) -> ChunkData<f32> {
    // todo, use mabey uninit
    let mut cdata = [0.0_f32; CHUNK_SQSIZE];
    
    for (idx, ptr) in cdata.iter_mut().enumerate() {
        let x = idx % CHUNK_SIZE;
        let y = idx / CHUNK_SIZE;
        
        let o = x + y * CHUNK_SIZE;
        let ox = (x+1) + y * CHUNK_SIZE;
        let oy = x + (y+1) * CHUNK_SIZE;
        let oxy = (x+1) + (y+1) * CHUNK_SIZE;
        
        if x != (CHUNK_SIZE-1) {
            if y != (CHUNK_SIZE-1) {
                let max_h = data[o].max(data[ox].max(data[oy].max(data[oxy])));
                let min_h = data[o].min(data[ox].min(data[oy].min(data[oxy])));
                let c = data[x+y*CHUNK_SIZE];
                
                *ptr = (c-max_h).abs().max((c-min_h).abs());
            }
        }
        
    }
    return Box::new(cdata)
}


/// generate a chunks texture, projected with (u, v) = (x, z)
// TODO consolidate arguments
pub fn chunktotexture(
    data:&ChunkData<f32>, 
    slopedata:&ChunkData<f32>, 
    _reg: &[reg::Regdata;4],
    grass : &Image, 
    water: &Image,
    sand: &Image,
    snow: &Image,
    stone: &Image,
    seed: (f32,f32)
) -> Image {
    let grass = grass.convert(TextureFormat::Rgba8UnormSrgb).unwrap();
    let ox = seed.0 as usize * (CHUNK_SIZE-1) * PIXELS_PER_POINT;
    let oy = seed.1 as usize * (CHUNK_SIZE-1) * PIXELS_PER_POINT;
    let tex = Image::new_fill(
        Extent3d {
            width: (PIXELS_PER_CHUNK) as u32,
            height: (PIXELS_PER_CHUNK) as u32,
            depth_or_array_layers: 1
        }
        ,TextureDimension::D2,
        &(0..(PIXELS_PER_CHUNK*PIXELS_PER_CHUNK))
            .flat_map(|i| {
                // position for texture
                let x = (i+ox)%PIXELS_PER_CHUNK;
                let y = (i+oy)/PIXELS_PER_CHUNK;
                
                // compute position in chunk (points)
                let chunk_xf = (i%PIXELS_PER_CHUNK) as f32 /PIXELS_PER_POINT as f32 ;
                let chunk_yf = (i/PIXELS_PER_CHUNK) as f32 /PIXELS_PER_POINT as f32 ;
                let chunk_x = (i%PIXELS_PER_CHUNK) as usize /PIXELS_PER_POINT as usize ;
                let chunk_y = (i/PIXELS_PER_CHUNK) as usize /PIXELS_PER_POINT as usize ;
                
                // the fractional part.
                let chunk_x_fraction = chunk_xf-chunk_xf.floor();
                let chunk_y_fraction = chunk_yf-chunk_yf.floor();
        
                assert!(chunk_x_fraction < 1.0);
                assert!(chunk_y_fraction < 1.0);
                
                //println!("{}",chunk_x_fraction);
        
                let point_ = chunk_x + chunk_y * CHUNK_SIZE;
                let point_x = (chunk_x+1) + chunk_y * CHUNK_SIZE;
                let point_y = chunk_x + (chunk_y+1) * CHUNK_SIZE;
                let point_xy = (chunk_x+1) + (chunk_y+1) * CHUNK_SIZE;
                
                let point_x = if point_xy < CHUNK_SQSIZE {point_x} else {point_};
                let point_y = if point_xy < CHUNK_SQSIZE {point_y} else {point_};
                let point_xy = if point_xy < CHUNK_SQSIZE {point_xy} else {point_};
                
                let h    = data[point_];
                let h_x  = data[point_x];
                let h_y  = data[point_y];
                let h_xy = data[point_xy];
                
                let ih = lerp(lerp(h_xy, h_y, chunk_x_fraction),lerp(h_x,h,chunk_x_fraction),chunk_y_fraction);
                
                 // compute texture index
                 let gidx = ((x%ASSET_SIZE)+(y%ASSET_SIZE)*ASSET_SIZE)*4;
                 
                 if ih > 22.0 {
                    return [snow.data[gidx + 0],snow.data[gidx + 1],snow.data[gidx + 2],255]
                 } else if ih > -0.3 {
                    if slopedata[point_] > 1.5 {
                         return[stone.data[gidx + 0],stone.data[gidx + 0],stone.data[gidx + 0],255]
                    } else {
                        return [grass.data[gidx + 0],grass.data[gidx + 1],grass.data[gidx + 2],255]
                    }
                 } else if ih> -0.7 {
                    return [sand.data[gidx + 0],sand.data[gidx + 1],sand.data[gidx + 2],255]
                 } else {
                     return [water.data[gidx + 0],water.data[gidx + 1],water.data[gidx + 2],255]
                 }
            })
            .collect::<Vec<u8>>()
        ,TextureFormat::Rgba8UnormSrgb
    );
    tex
}

/// convert a hightmap to a mesh with (u, v) = (x, z) texture maping
/// note, if you intend to tile the meshes the hightmaps must have 1 sample of overlap.
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
        let x = i % CHUNK_SIZE;
        let y = i / CHUNK_SIZE;
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
        let m0 = n + Vector3::from_row_slice(&normals[idx0]);
        let m1 = n + Vector3::from_row_slice(&normals[idx1]);
        let m2 = n + Vector3::from_row_slice(&normals[idx2]);
        
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
pub fn gen(assets: &mut Assets<Image>,seed: (f32,f32)) -> Result<(Image,Mesh,ChunkData<f32>),String> {
    /// grab assets from ecs
    let grass = assets.get(ASSETS_GRASS).map_or_else(|| Err("cant get grass.".to_string()), |x| Ok(x))?;
    let water = assets.get(ASSETS_WATER).map_or_else(|| Err("cant get water.".to_string()), |x| Ok(x))?;
    let sand = assets.get(ASSETS_SAND).map_or_else(|| Err("cant get sand.".to_string()), |x| Ok(x))?;
    let snow = assets.get(ASSETS_SNOW).map_or_else(|| Err("cant get snow.".to_string()), |x| Ok(x))?;
    let stone = assets.get(ASSETS_STONE).map_or_else(|| Err("cant get stone.".to_string()), |x| Ok(x))?;
    // generate region data, this could be optimized
    let reg   = genchunkreg(seed);
    let regx  = genchunkreg((seed.0 + 1.0, seed.1 + 0.0));
    let regy  = genchunkreg((seed.0 + 0.0, seed.1 + 1.0));
    let regxy = genchunkreg((seed.0 + 1.0, seed.1 + 1.0));
    let regs = [reg,regx,regy,regxy];
    // generate the hightmap
    let hightmap = genchunk(seed,&regs);
    // compute slope
    let slopemap = genslope(&hightmap);
    // create mesh
    let mesh = chunktomesh(&hightmap);
    let tex = chunktotexture(
        &hightmap,
        &slopemap,
        &regs,
        grass,
        water,
        sand,
        snow,
        stone,
        seed
    );
    Ok((tex,mesh,hightmap))
}
