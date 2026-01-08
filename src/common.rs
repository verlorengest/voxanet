//common.rs

use std::collections::{HashMap, HashSet};
use bytemuck::{Pod, Zeroable};
use crate::noise::PlanetTerrain;

// --- CONSTANTS ---
pub const CHUNK_SIZE: u32 = 32;

// --- DATA TYPES ---

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct BlockId {
    pub face: u8, 
    pub layer: u32, 
    pub u: u32, 
    pub v: u32,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ChunkKey {
    pub face: u8, 
    pub u_idx: u32, 
    pub v_idx: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

pub struct ChunkMesh {
    pub v_buf: wgpu::Buffer,
    pub i_buf: wgpu::Buffer,
    pub num_inds: u32,
    pub num_verts: usize,
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub center: glam::Vec3,
    pub radius: f32,
}



#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct LodKey {
    pub face: u8,
    pub x: u32,      
    pub y: u32,      
    pub size: u32,   
}


#[derive(Clone)] 
pub struct ChunkMods {
    pub mined: HashSet<BlockId>,
    pub placed: HashSet<BlockId>,
}



impl ChunkMods {
    pub fn new() -> Self {
        Self { mined: HashSet::new(), placed: HashSet::new() }
    }
}

#[derive(Clone)] 
pub struct PlanetData {
    pub chunks: HashMap<ChunkKey, ChunkMods>, 
    pub resolution: u32,
    pub has_core: bool,
    pub terrain: crate::noise::PlanetTerrain,
}

impl PlanetData {
    pub fn new(resolution: u32) -> Self {
        println!("Generating Terrain Noise Map for res {}...", resolution);
        let terrain = PlanetTerrain::new(resolution); // calculate once
        println!("Terrain Generation Complete.");
        
        Self {
            chunks: HashMap::new(),
            resolution,
            has_core: true,
            terrain, // <--- Store it
        }
    }

pub fn resize(&mut self, increase: bool) {
        if increase {
            // multiply by 1.2
            // i use .max(self.resolution + 1) to ensure it always grows by at least 1 block
            let new_res = (self.resolution as f32 * 1.2) as u32;
            self.resolution = new_res.max(self.resolution + 1).min(16384); 
        } else {
            // divide by 1.2
            let new_res = (self.resolution as f32 / 1.2) as u32;
            self.resolution = new_res.max(8);
        }
        

        self.chunks.clear();
        
        // regenerate noise map for new resolution
        println!("Regenerating Terrain for new res {}...", self.resolution);
        self.terrain = PlanetTerrain::new(self.resolution); 
    }

    fn get_chunk_key(id: BlockId) -> ChunkKey {
        ChunkKey {
            face: id.face,
            u_idx: id.u / CHUNK_SIZE,
            v_idx: id.v / CHUNK_SIZE,
        }
    }

    pub fn add_block(&mut self, id: BlockId) {
        let key = Self::get_chunk_key(id);
        let mods = self.chunks.entry(key).or_insert_with(ChunkMods::new);
        
        if mods.mined.contains(&id) {
            mods.mined.remove(&id);
        } else {
            mods.placed.insert(id);
        }
    }

pub fn remove_block(&mut self, id: BlockId) {
        // protect the bottom 4 layers as the unbreakable core
        if self.has_core && id.layer < 6 {
            return; 
        }
        
        let key = Self::get_chunk_key(id);
        let mods = self.chunks.entry(key).or_insert_with(ChunkMods::new);

        if mods.placed.contains(&id) {
            mods.placed.remove(&id);
        } else {
            if id.layer < self.resolution {
                mods.mined.insert(id);
            }
        }
    }
    
    pub fn exists(&self, id: BlockId) -> bool {
        let key = Self::get_chunk_key(id);
        if let Some(mods) = self.chunks.get(&key) {
            if mods.placed.contains(&id) { return true; }
            if mods.mined.contains(&id) { return false; }
        }
        

        // instead of a flat floor, we check the pre-calculated noise map
        let height = self.terrain.get_height(id.face, id.u, id.v);
        id.layer <= height
    }

    
}


// --- FRUSTUM CULLING HELPER ---

pub struct Frustum {
    planes: [glam::Vec4; 6],
}

impl Frustum {
    pub fn from_matrix(m: glam::Mat4) -> Self {
        let r0 = m.row(0);
        let r1 = m.row(1);
        let r2 = m.row(2);
        let r3 = m.row(3);

        let mut planes = [
            r3 + r0, // Left
            r3 - r0, // Right
            r3 + r1, // Bottom
            r3 - r1, // Top
            r3 + r2, // Near
            r3 - r2, // Far
        ];

        // normalize planes
        for plane in &mut planes {
            let len = glam::Vec3::new(plane.x, plane.y, plane.z).length();
            *plane /= len;
        }

        Self { planes }
    }

    // returns true if a sphere is partly or fully inside the frustum
    pub fn intersects_sphere(&self, center: glam::Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let dist = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            
            if dist < -radius {
                return false;
            }
        }
        true
    }
}