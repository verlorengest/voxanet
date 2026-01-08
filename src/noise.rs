use glam::Vec3;
use crate::gen::CoordSystem;
use std::sync::Arc; 

// --- SETTINGS & ENUMS ---

#[derive(Clone, Copy, Debug)]
pub enum NoiseType {
    Perlin,
    Simplex,  
    Cellular, 
}

#[derive(Clone, Copy, Debug)]
pub struct NoiseSettings {
    pub noise_type: NoiseType,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32, 
    pub lacunarity: f32,  
    pub offset: Vec3,     
}

impl NoiseSettings {
   
    pub fn default_terrain(res: u32) -> Self {
        Self {
            noise_type: NoiseType::Perlin,
            frequency: res as f32 / 100.0, 
            amplitude: 24.0,
            octaves: 4,      
            persistence: 0.5,
            lacunarity: 2.0,
            offset: Vec3::ZERO,
        }
    }
}

// --- PLANET TERRAIN DATA ---

pub struct PlanetTerrain {
    // Flattened height map
    heights: Arc<Vec<u16>>, 
    resolution: u32,
}

impl PlanetTerrain {
    pub fn new(resolution: u32) -> Self {
        let size = (6 * resolution * resolution) as usize;
        let mut heights = vec![0; size];
        let generator = NoiseGenerator::new(42); // Seed 42
        let settings = NoiseSettings::default_terrain(resolution);
        let base_radius = resolution as f32 / 2.0;
        for face in 0..6 {
            for v in 0..resolution {
                for u in 0..resolution {
                    let dir = CoordSystem::get_direction(face, u, v, resolution);
                    let noise_val = generator.compute(dir, &settings);
                    let h_offset = noise_val * settings.amplitude;
                    let final_layer = (base_radius + h_offset).max(1.0) as u16;
                    let idx = Self::get_index(face, u, v, resolution);
                    heights[idx] = final_layer;
                }
            }
        }

        // Wrap in Arc for cheap cloning
        Self { heights: Arc::new(heights), resolution } 
    }

    #[inline(always)]
    fn get_index(face: u8, u: u32, v: u32, res: u32) -> usize {
        let face_offset = (face as usize) * (res as usize) * (res as usize);
        let row_offset = (v as usize) * (res as usize);
        face_offset + row_offset + (u as usize)
    }

    pub fn get_height(&self, face: u8, u: u32, v: u32) -> u32 {
        let u_safe = u.min(self.resolution - 1);
        let v_safe = v.min(self.resolution - 1);

        let idx = Self::get_index(face, u_safe, v_safe, self.resolution);
        self.heights[idx] as u32
    }
    
    }

impl Clone for PlanetTerrain {
    fn clone(&self) -> Self {
        Self {
            heights: self.heights.clone(),
            resolution: self.resolution,
        }
    }
}


// --- NOISE GENERATOR ---

struct NoiseGenerator {
    perm: [u8; 512],
}

impl NoiseGenerator {
    fn new(seed: u32) -> Self {
        let mut p = [0u8; 512];
        let mut permutation: Vec<u8> = (0..=255).collect();
        let mut state = seed;
        for i in (1..256).rev() {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (state as usize) % (i + 1);
            permutation.swap(i, j);
        }

        for i in 0..256 {
            p[i] = permutation[i];
            p[i + 256] = permutation[i];
        }
        Self { perm: p }
    }

    fn compute(&self, pos: Vec3, settings: &NoiseSettings) -> f32 {
        if settings.octaves <= 1 {
            let p = pos * settings.frequency + settings.offset;
            return self.compute_base(p, settings.noise_type); // Returns 0..1
        }

        let mut total_val = 0.0;
        let mut total_amp = 0.0;
        
        let mut amp = 1.0;
        let mut freq = settings.frequency;
        let mut p = pos;

        for _ in 0..settings.octaves {
            let sample_pos = p * freq + settings.offset;
            total_val += self.compute_base(sample_pos, settings.noise_type) * amp;
            total_amp += amp;

            amp *= settings.persistence;
            freq *= settings.lacunarity;
        }

        // normalize result to 0..1 range
        if total_amp > 0.0 {
            total_val / total_amp
        } else {
            0.0
        }
    }

    fn compute_base(&self, p: Vec3, type_: NoiseType) -> f32 {
        match type_ {
            NoiseType::Perlin => {
                (self.perlin(p) + 1.0) * 0.5
            },
            NoiseType::Simplex => 0.0, // TODO: implement simplex
            NoiseType::Cellular => 0.0, // TODO: implement cellular
        }
    }

    // --- PERLIN MATH ---
    
    fn perlin(&self, pos: Vec3) -> f32 {
        let x = pos.x.floor();
        let y = pos.y.floor();
        let z = pos.z.floor();
        
        let X = x as i32 & 255;
        let Y = y as i32 & 255;
        let Z = z as i32 & 255;

        let x = pos.x - x;
        let y = pos.y - y;
        let z = pos.z - z;

        let u = fade(x);
        let v = fade(y);
        let w = fade(z);

        let A = self.perm[X as usize] as usize + Y as usize;
        let AA = self.perm[A] as usize + Z as usize;
        let AB = self.perm[A + 1] as usize + Z as usize;
        let B = self.perm[X as usize + 1] as usize + Y as usize;
        let BA = self.perm[B] as usize + Z as usize;
        let BB = self.perm[B + 1] as usize + Z as usize;

        lerp(w, lerp(v, lerp(u, grad(self.perm[AA], x, y, z),
                                grad(self.perm[BA], x - 1.0, y, z)),
                        lerp(u, grad(self.perm[AB], x, y - 1.0, z),
                                grad(self.perm[BB], x - 1.0, y - 1.0, z))),
                lerp(v, lerp(u, grad(self.perm[AA + 1], x, y, z - 1.0),
                                grad(self.perm[BA + 1], x - 1.0, y, z - 1.0)),
                        lerp(u, grad(self.perm[AB + 1], x, y - 1.0, z - 1.0),
                                grad(self.perm[BB + 1], x - 1.0, y - 1.0, z - 1.0))))
    }
}

// ---MATH-HELPERS---

fn fade(t: f32) -> f32 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
fn lerp(t: f32, a: f32, b: f32) -> f32 { a + t * (b - a) }
fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else { if h == 12 || h == 14 { x } else { z } };
    (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
}