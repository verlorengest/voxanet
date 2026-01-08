//gen.rs

use std::collections::HashSet;
use glam::Vec3;
use crate::common::*;

pub struct CoordSystem;

impl CoordSystem {
    // k = 0.85 balances the shape.
    const K: f64 = 0.85; 


// forward Mapping: Unit Cube -> Sphere
    fn cube_to_sphere(x: f64, y: f64, z: f64) -> Vec3 {
        let x2 = x * x;
        let y2 = y * y;
        let z2 = z * z;

        let sx = x * (1.0 - y2 * 0.5 - z2 * 0.5 + y2 * z2 / 3.0).sqrt();
        let sy = y * (1.0 - z2 * 0.5 - x2 * 0.5 + z2 * x2 / 3.0).sqrt();
        let sz = z * (1.0 - x2 * 0.5 - y2 * 0.5 + x2 * y2 / 3.0).sqrt();
        
        Vec3::new(sx as f32, sy as f32, sz as f32)
    }

    // inverse Mapping: Sphere -> Unit Cube
    
    fn cubize_point(pos: Vec3) -> Vec3 {
        let mut x = pos.x as f64;
        let mut y = pos.y as f64;
        let mut z = pos.z as f64;

        let fx = x.abs();
        let fy = y.abs();
        let fz = z.abs();

        const INVERSE_SQRT_2: f64 = 0.70710676908493042;

        if fy >= fx && fy >= fz {
            let a2 = x * x * 2.0;
            let b2 = z * z * 2.0;
            let inner = -a2 + b2 - 3.0;
            let inner_sqrt = -((inner * inner) - 12.0 * a2).sqrt();

            if x == 0.0 { x = 0.0; } 
            else { x = (inner_sqrt + a2 - b2 + 3.0).sqrt() * INVERSE_SQRT_2; }

            if z == 0.0 { z = 0.0; } 
            else { z = (inner_sqrt - a2 + b2 + 3.0).sqrt() * INVERSE_SQRT_2; }

            if x > 1.0 { x = 1.0; }
            if z > 1.0 { z = 1.0; }

            if pos.x < 0.0 { x = -x; }
            if pos.z < 0.0 { z = -z; }

            y = if pos.y > 0.0 { 1.0 } else { -1.0 };
        } else if fx >= fy && fx >= fz {
            let a2 = y * y * 2.0;
            let b2 = z * z * 2.0;
            let inner = -a2 + b2 - 3.0;
            let inner_sqrt = -((inner * inner) - 12.0 * a2).sqrt();

            if y == 0.0 { y = 0.0; } 
            else { y = (inner_sqrt + a2 - b2 + 3.0).sqrt() * INVERSE_SQRT_2; }

            if z == 0.0 { z = 0.0; } 
            else { z = (inner_sqrt - a2 + b2 + 3.0).sqrt() * INVERSE_SQRT_2; }

            if y > 1.0 { y = 1.0; }
            if z > 1.0 { z = 1.0; }

            if pos.y < 0.0 { y = -y; }
            if pos.z < 0.0 { z = -z; }

            x = if pos.x > 0.0 { 1.0 } else { -1.0 };
        } else {
            let a2 = x * x * 2.0;
            let b2 = y * y * 2.0;
            let inner = -a2 + b2 - 3.0;
            let inner_sqrt = -((inner * inner) - 12.0 * a2).sqrt();

            if x == 0.0 { x = 0.0; } 
            else { x = (inner_sqrt + a2 - b2 + 3.0).sqrt() * INVERSE_SQRT_2; }

            if y == 0.0 { y = 0.0; } 
            else { y = (inner_sqrt - a2 + b2 + 3.0).sqrt() * INVERSE_SQRT_2; }

            if x > 1.0 { x = 1.0; }
            if y > 1.0 { y = 1.0; }

            if pos.x < 0.0 { x = -x; }
            if pos.y < 0.0 { y = -y; }

            z = if pos.z > 0.0 { 1.0 } else { -1.0 };
        }
        Vec3::new(x as f32, y as f32, z as f32)
    }





    pub fn get_local_coords(pos: Vec3, res: u32) -> Option<(BlockId, Vec3)> {
        let dist = pos.length() as f64;
        let s = res as f64 / 2.0;
        
        let min_r = s * (-Self::K).exp(); 
        if dist < min_r { return None; }

        let layer_f = s * (1.0 + (dist / s).ln() / Self::K);
        let layer = layer_f.floor() as i32;
        
        if layer < 0 || layer >= res as i32 { return None; }
        
        // local Layer Coordinate (0.0 to 1.0)
        let f_layer = (layer_f - layer as f64) as f32;

        // map sphere point back to Unit Cube
        let cube_pos = Self::cubize_point(pos.normalize());
        let abs = cube_pos.abs();
        
        let (face, u_local, v_local) = if abs.y >= abs.x && abs.y >= abs.z {
            if cube_pos.y > 0.0 { (0, cube_pos.x, cube_pos.z) } else { (1, cube_pos.x, cube_pos.z) }
        } else if abs.x >= abs.y && abs.x >= abs.z {
            if cube_pos.x > 0.0 { (2, cube_pos.y, cube_pos.z) } else { (3, cube_pos.y, cube_pos.z) }
        } else {
            if cube_pos.z > 0.0 { (4, cube_pos.x, cube_pos.y) } else { (5, cube_pos.x, cube_pos.y) }
        };

        let rf = res as f64;
        
        // calculate raw grid coordinates
        let u_raw = (u_local as f64 * rf + rf) / 2.0;
        let v_raw = (v_local as f64 * rf + rf) / 2.0;
        
        let u = u_raw.floor() as i32;
        let v = v_raw.floor() as i32;

        // local UV Coordinates (0.0 to 1.0)
        let f_u = (u_raw - u as f64) as f32;
        let f_v = (v_raw - v as f64) as f32;

        let u = u.clamp(0, res as i32 - 1) as u32;
        let v = v.clamp(0, res as i32 - 1) as u32;

        Some((
            BlockId { face: face as u8, layer: layer as u32, u, v },
            Vec3::new(f_u, f_v, f_layer) // x=u, y=v, z=layer
        ))
    }




    pub fn get_layer_radius(layer: u32, res: u32) -> f32 {
        let s = res as f64 / 2.0;
        let r = s * (Self::K * ((layer as f64 / s) - 1.0)).exp();
        r as f32
    }

pub fn get_direction(face: u8, u: u32, v: u32, res: u32) -> Vec3 {
        let rf = res as f64;
        
        let x_local = if u == 0 { -1.0 } else if u == res { 1.0 } else { 
            (u as f64 * 2.0 - rf) / rf
        };
        
        let y_local = if v == 0 { -1.0 } else if v == res { 1.0 } else { 
            (v as f64 * 2.0 - rf) / rf
        };
        
        let (cx, cy, cz) = match face {
            0 => (x_local, 1.0, y_local),  
            1 => (x_local, -1.0, y_local),
            2 => (1.0, x_local, y_local),  
            3 => (-1.0, x_local, y_local),
            4 => (x_local, y_local, 1.0),  
            _ => (x_local, y_local, -1.0),
        };

        Self::cube_to_sphere(cx, cy, cz).normalize()
    }

    pub fn get_vertex_pos(face: u8, u: u32, v: u32, layer: u32, res: u32) -> Vec3 {
        let dir = Self::get_direction(face, u, v, res);
        let radius = Self::get_layer_radius(layer, res);
        dir * radius
    }

    pub fn get_block_center(face: u8, u: u32, v: u32, layer: u32, res: u32) -> Vec3 {
        let rf = res as f64;
        // center is at index + 0.5
        let uf = u as f64 + 0.5;
        let vf = v as f64 + 0.5;
        
        let x_local = (uf * 2.0 - rf) / rf;
        let y_local = (vf * 2.0 - rf) / rf;
        
        let (cx, cy, cz) = match face {
            0 => (x_local, 1.0, y_local),  
            1 => (x_local, -1.0, y_local),
            2 => (1.0, x_local, y_local),  
            3 => (-1.0, x_local, y_local),
            4 => (x_local, y_local, 1.0),  
            _ => (x_local, y_local, -1.0),
        };

        let dir = Self::cube_to_sphere(cx, cy, cz).normalize();

        let s = rf / 2.0;
        let radius = s * (Self::K * (((layer as f64 + 0.5) / s) - 1.0)).exp();
        
        dir * (radius as f32)
    }

pub fn pos_to_id(pos: Vec3, res: u32) -> Option<BlockId> {
        let dist = pos.length() as f64;
        let s = res as f64 / 2.0;
        
        let min_r = s * (-Self::K).exp(); 
        if dist < min_r { return None; }

        let layer_f = s * (1.0 + (dist / s).ln() / Self::K);
        let layer = layer_f.floor() as i32;

        if layer < 0 { return None; }
        let layer = layer as u32;
        if layer >= res { return None; }

        // map sphere point back to unit cube surface
        // normalize 'pos' first to project it onto the unit sphere required for the math
        let cube_pos = Self::cubize_point(pos.normalize());
        
        // determine Face based on which component is 1.0 or -1.0
        // use a small epsilon for float comparison safety, though logic forces exactly 1.0
        let abs = cube_pos.abs();
        let (face, u_local, v_local) = if abs.y >= abs.x && abs.y >= abs.z {
            if cube_pos.y > 0.0 { (0, cube_pos.x, cube_pos.z) } else { (1, cube_pos.x, cube_pos.z) }
        } else if abs.x >= abs.y && abs.x >= abs.z {
            if cube_pos.x > 0.0 { (2, cube_pos.y, cube_pos.z) } else { (3, cube_pos.y, cube_pos.z) }
        } else {
            if cube_pos.z > 0.0 { (4, cube_pos.x, cube_pos.y) } else { (5, cube_pos.x, cube_pos.y) }
        };

        // convert Local [-1, 1] coords to grid indices
        let rf = res as f64;
        // x = (u * 2 - res) / res  =>  u = (x * res + res) / 2
        let u_raw = ((u_local as f64 * rf + rf) / 2.0).floor() as i32;
        let v_raw = ((v_local as f64 * rf + rf) / 2.0).floor() as i32;

        let u = u_raw.clamp(0, res as i32 - 1) as u32;
        let v = v_raw.clamp(0, res as i32 - 1) as u32;

        Some(BlockId { face: face as u8, layer, u, v })
    }
}

pub struct MeshGen;

impl MeshGen {

    fn add_mined_candidates(mods: &ChunkMods, candidates: &mut HashSet<BlockId>, res: u32) {
        for &id in &mods.mined {
            candidates.insert(BlockId { layer: id.layer + 1, ..id });
            if id.layer > 0 { candidates.insert(BlockId { layer: id.layer - 1, ..id }); }
            if id.u > 0 { candidates.insert(BlockId { u: id.u - 1, ..id }); }
            if id.u < res - 1 { candidates.insert(BlockId { u: id.u + 1, ..id }); }
            if id.v > 0 { candidates.insert(BlockId { v: id.v - 1, ..id }); }
            if id.v < res - 1 { candidates.insert(BlockId { v: id.v + 1, ..id }); }
        }
    }

    pub fn build_chunk(key: ChunkKey, data: &PlanetData) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;
        let res = data.resolution;
        let mut candidates = HashSet::new();

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        // Ensure we don't iterate past resolution even if key exists
        let u_end = (u_start + CHUNK_SIZE).min(res); 
        let v_end = (v_start + CHUNK_SIZE).min(res);

        // natural Surface (with slope filling)
        // need to check neighbors to see how far down the cliff goes.
        // if a neighbor is lower than us, we must generate the blocks between our height and theirs.
        
        // safely get height from the terrain map
        let get_h = |f, u, v| -> u32 {
             if u >= res || v >= res { return 0; } 
             // using 0 here means "very deep", so we might generate extra mesh at face edges, which is safer than holes.
             data.terrain.get_height(f, u, v)
        };

        for u in u_start..u_end {
            for v in v_start..v_end {
                let h = get_h(key.face, u, v);
                if h == 0 { continue; }

                // always add the top surface block
                candidates.insert(BlockId { face: key.face, layer: h, u, v });

                // check immediate neighbors to find the lowest exposed point
                let mut min_h = h;
                
                if u > 0 { min_h = min_h.min(get_h(key.face, u - 1, v)); }
                if u < res - 1 { min_h = min_h.min(get_h(key.face, u + 1, v)); }
                if v > 0 { min_h = min_h.min(get_h(key.face, u, v - 1)); }
                if v < res - 1 { min_h = min_h.min(get_h(key.face, u, v + 1)); }

                if min_h < h {
                    let bottom = min_h.max(h.saturating_sub(20)); 
                    
                    for l in (bottom + 1)..h {
                         candidates.insert(BlockId { face: key.face, layer: l, u, v });
                    }
                }
            }
        }

        // current Chunk Modifications
        if let Some(mods) = data.chunks.get(&key) {
            for &id in &mods.placed { candidates.insert(id); }
            Self::add_mined_candidates(mods, &mut candidates, res);
        }

        // neighbor Chunks Modifications 
        let neighbor_keys = [
            ChunkKey { u_idx: key.u_idx.wrapping_sub(1), ..key },
            ChunkKey { u_idx: key.u_idx + 1, ..key },
            ChunkKey { v_idx: key.v_idx.wrapping_sub(1), ..key },
            ChunkKey { v_idx: key.v_idx + 1, ..key },
        ];

        for n_key in neighbor_keys {
            if let Some(mods) = data.chunks.get(&n_key) {
                Self::add_mined_candidates(mods, &mut candidates, res);
            }
        }

        // generate Mesh
        for id in candidates {
            if id.u >= u_start && id.u < u_end && id.v >= v_start && id.v < v_end {
                if data.exists(id) {
                    Self::add_voxel(id, data, &mut verts, &mut inds, &mut idx);
                }
            }
        }
        (verts, inds)
    }


    // side1, side2: the two blocks flanking the vertex
    // corner: the block diagonally connecting the vertex
    fn calculate_ao(side1: bool, side2: bool, corner: bool) -> f32 {
        let mut occ = 0;
        if side1 { occ += 1; }
        if side2 { occ += 1; }
        if corner && (side1 || side2) { occ += 1; }
        
        // 0=Bright, 1=Dim, 2=Dark, 3=Very Dark
        match occ {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.4,
        }
    }




// Generates wireframe boxes for collision detection debugging
    pub fn generate_collision_debug(player_pos: Vec3, planet: &PlanetData) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let res = planet.resolution;
        let color = [1.0, 0.0, 0.0]; // red
        let normal = [0.0, 1.0, 0.0];

        // check a 3x3x3 area around the player
        let range = 2; 
        
        if let Some((center_id, _)) = CoordSystem::get_local_coords(player_pos, res) {
            let start_u = (center_id.u as i32 - range).max(0);
            let end_u = (center_id.u as i32 + range).min(res as i32 - 1);
            let start_v = (center_id.v as i32 - range).max(0);
            let end_v = (center_id.v as i32 + range).min(res as i32 - 1);
            let start_l = (center_id.layer as i32 - range).max(0);
            let end_l = (center_id.layer as i32 + range).min(res as i32 - 1);

            let mut idx = 0;

            for l in start_l..=end_l {
                for v in start_v..=end_v {
                    for u in start_u..=end_u {
                        let id = crate::common::BlockId { face: center_id.face, layer: l as u32, u: u as u32, v: v as u32 };
                        

                        let block_pos = CoordSystem::get_block_center(id.face, id.u, id.v, id.layer, res);
                        
                        if crate::physics::Physics::is_solid(block_pos, planet) {
                            // visualize the "Core" of the block that triggers collision
                            let get_p = |uu, vv, ll| {
                                CoordSystem::get_vertex_pos(id.face, id.u + uu, id.v + vv, id.layer + ll, res)
                            };

                            // get corners of the voxel
                            let c000 = get_p(0,0,0); let c100 = get_p(1,0,0);
                            let c010 = get_p(0,1,0); let c110 = get_p(1,1,0);
                            let c001 = get_p(0,0,1); let c101 = get_p(1,0,1);
                            let c011 = get_p(0,1,1); let c111 = get_p(1,1,1);

                            // shrink corners towards center by margin (visualize the "shave")
                            let center = (c000+c100+c010+c110+c001+c101+c011+c111) * 0.125;
                            let shrink = 0.90; // Exaggerate the shrink slightly so we can see it inside the block
                            
                            let v = |p: Vec3| Vertex { pos: (center + (p - center) * shrink).to_array(), color, normal };
                            
                            let corners = [
                                v(c000), v(c100), v(c110), v(c010), // Bottom
                                v(c001), v(c101), v(c111), v(c011)  // Top
                            ];

                            // add vertices
                            for c in &corners { verts.push(*c); }

                            // add line indices (Cube wireframe)
                            let base = idx;
                            let lines = [
                                (0,1), (1,2), (2,3), (3,0), // Bottom ring
                                (4,5), (5,6), (6,7), (7,4), // Top ring
                                (0,4), (1,5), (2,6), (3,7)  // Pillars
                            ];

                            for (s, e) in lines {
                                inds.push(base + s); inds.push(base + e);
                            }
                            idx += 8;
                        }
                    }
                }
            }
        }
        (verts, inds)
    }




    // generates a simplified heightmap mesh for distant terrain
    pub fn generate_lod_mesh(key: crate::common::LodKey, data: &PlanetData) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        
      
        let grid_res = 64; 
        let row_len = grid_res + 1;
        
        // calculate global pos for any grid index (even outside this chunk)
        // this allows us to "peek" into neighbor chunks for perfect normals.
        let get_sample_pos = |gx: i32, gy: i32| -> glam::Vec3 {
            
             let step_u = (gx as i64 * key.size as i64) / grid_res as i64;
             let step_v = (gy as i64 * key.size as i64) / grid_res as i64;
             
             // calculate absolute U/V
             let abs_u = (key.x as i64 + step_u).clamp(0, data.resolution as i64) as u32;
             let abs_v = (key.y as i64 + step_v).clamp(0, data.resolution as i64) as u32;
             
             let h = data.terrain.get_height(key.face, abs_u, abs_v);
             CoordSystem::get_vertex_pos(key.face, abs_u, abs_v, h, data.resolution)
        };

        // 1. Generate Vertices
        for vy in 0..=grid_res {
            for ux in 0..=grid_res {
                let pos = get_sample_pos(ux as i32, vy as i32);

                // seamless normal fix
                // instead of clamping to grid edges, we look -1 and +1 in global grid Space
                // this ensures the normal at the chunk edge matches the neighbor's normal perfectly
                
                let p_right = get_sample_pos(ux as i32 + 1, vy as i32);
                let p_left  = get_sample_pos(ux as i32 - 1, vy as i32);
                let p_down  = get_sample_pos(ux as i32, vy as i32 + 1);
                let p_up    = get_sample_pos(ux as i32, vy as i32 - 1);
                
                // central Difference
                let tangent_u = p_right - p_left;
                let tangent_v = p_down - p_up;

                let mut normal = tangent_u.cross(tangent_v).normalize();
                if normal.dot(pos.normalize()) < 0.0 { normal = -normal; }

                // --- COLORING ---
                let slope = normal.dot(pos.normalize()).abs();
                
                // recalculate h locally for core check
                let offset_u = (ux * key.size) / grid_res;
                let offset_v = (vy * key.size) / grid_res;
                let h = data.terrain.get_height(key.face, (key.x + offset_u).min(data.resolution), (key.y + offset_v).min(data.resolution));
                
                let is_core = data.has_core && h < 6;
                let is_steep = slope < 0.85; 

                let color = if is_core { 
                    [0.2, 0.22, 0.25] 
                } else if is_steep { 
                    [0.1 * 0.75, 0.8 * 0.75, 0.1 * 0.75] // Dark Green (Matches Voxel Sides)
                } else { 
                    [0.1, 0.8, 0.1]    // Green (Top)
                };

                verts.push(Vertex { pos: pos.to_array(), color, normal: normal.to_array() });
            }
        }

        // generate indices
        for y in 0..grid_res {
            for x in 0..grid_res {
                let tl = y * row_len + x;
                let tr = tl + 1;
                let bl = (y + 1) * row_len + x;
                let br = bl + 1;

                inds.push(tl); inds.push(bl); inds.push(tr);
                inds.push(tr); inds.push(bl); inds.push(br);
            }
        }

        // generate Skirts (hides physical gaps)
        let radius = CoordSystem::get_layer_radius(data.resolution / 2, data.resolution);
        let chunk_phys_size = (key.size as f32 / data.resolution as f32) * radius; 
        
        
        let skirt_depth = (chunk_phys_size * 0.15).clamp(4.0, 500.0);

        let mut add_skirt_edge = |coord_pairs: &[(u32, u32)], reverse: bool| {
            let base_idx = verts.len() as u32;
            for &(ux, vy) in coord_pairs {
                let src_idx = vy * row_len + ux;
                let src_v = verts[src_idx as usize];
                
                // bend skirt inwards slightly to avoid poking through other meshes
                let p = glam::Vec3::from_array(src_v.pos);
                let down = -p.normalize() * skirt_depth;
                
                verts.push(Vertex { pos: (p + down).to_array(), color: src_v.color, normal: src_v.normal });
            }
            let len = coord_pairs.len() as u32;
            for i in 0..(len - 1) {
                let s1 = coord_pairs[i as usize].1 * row_len + coord_pairs[i as usize].0;
                let s2 = coord_pairs[(i + 1) as usize].1 * row_len + coord_pairs[(i + 1) as usize].0;
                let k1 = base_idx + i;
                let k2 = base_idx + i + 1;
                
                // winding
                if reverse {
                     inds.push(s1); inds.push(k2); inds.push(k1);
                     inds.push(s1); inds.push(s2); inds.push(k2);
                } else {
                     inds.push(s1); inds.push(k1); inds.push(k2);
                     inds.push(s1); inds.push(k2); inds.push(s2);
                }
            }
        };

        // define active edges positive logic
        let top: Vec<(u32, u32)> = (0..=grid_res).map(|x| (x, 0)).collect();
        let bottom: Vec<(u32, u32)> = (0..=grid_res).map(|x| (x, grid_res)).collect();
        let left: Vec<(u32, u32)> = (0..=grid_res).map(|y| (0, y)).collect();
        let right: Vec<(u32, u32)> = (0..=grid_res).map(|y| (grid_res, y)).collect();

        add_skirt_edge(&top, false);
        add_skirt_edge(&bottom, true);
        add_skirt_edge(&left, true);
        add_skirt_edge(&right, false);

        (verts, inds)
    }

fn add_voxel(id: BlockId, data: &PlanetData, verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, idx: &mut u32) {
        let res = data.resolution;

        // neighbor existence check
        let check = |d_face: u8, d_layer: i32, d_u: i32, d_v: i32| -> bool {
            let l = id.layer as i32 + d_layer;
            let u = id.u as i32 + d_u;
            let v = id.v as i32 + d_v;
            if l >= 0 && u >= 0 && u < res as i32 && v >= 0 && v < res as i32 {
                return data.exists(BlockId { face: d_face, layer: l as u32, u: u as u32, v: v as u32 });
            }
            l < 0 // Core is solid
        };

        // --- FACE CHECKS ---
        let has_top   = check(id.face, 1, 0, 0);
        let has_btm   = check(id.face, -1, 0, 0);
        let has_right = check(id.face, 0, 1, 0);
        let has_left  = check(id.face, 0, -1, 0);
        let has_back  = check(id.face, 0, 0, 1);
        let has_front = check(id.face, 0, 0, -1);

        if has_top && has_btm && has_left && has_right && has_front && has_back { return; }

        // --- LIGHTING CALCULATION ( this is simple, i will change this later)---
        // we cast a short ray (8 blocks)
        // if we hit nothing, we assume we are near the surface
        // if we hit blocks, we darken

        let mut sky_occlusion: f32 = 0.0; 
        for i in 1..=8 {
            if check(id.face, i, 0, 0) {
                sky_occlusion += 1.0;
            }
        }
        // 0.0 = full sky, 1.0 = buried

        let mut light_val: f32 = 1.0; 
        
        for i in 1..=8 {
            if check(id.face, i, 0, 0) {
                light_val = 0.15; // Dark shadow immediately
                break;
            }
        }

        // boost light if it's the natural surface (Grass) to ensure terrain looks bright
        let natural_h = data.terrain.get_height(id.face, id.u, id.v);
        if id.layer >= natural_h { light_val = 1.0; }

     
        let is_core = data.has_core && id.layer < 6;
        let is_grass = id.layer == natural_h;
        
        let mut base_color = if is_core { 
            [0.2, 0.2, 0.2] // rock
        } else if is_grass { 
            [0.1, 0.7, 0.1] // grass
        } else { 
            [0.6, 0.4, 0.2] // dirt
        };

        // apply Skylight
        base_color[0] *= light_val;
        base_color[1] *= light_val;
        base_color[2] *= light_val;

        // geometry Helpers
        let p = |u_off: u32, v_off: u32, l_off: u32| CoordSystem::get_vertex_pos(id.face, id.u + u_off, id.v + v_off, id.layer + l_off, res);
        let i_bl = p(0,0,0); let i_br = p(1,0,0); let i_tl = p(0,1,0); let i_tr = p(1,1,0);
        let o_bl = p(0,0,1); let o_br = p(1,0,1); let o_tl = p(0,1,1); let o_tr = p(1,1,1);

        let apply = |ao: f32| -> [f32; 3] { [base_color[0] * ao, base_color[1] * ao, base_color[2] * ao] };

   
        if !has_top {
            
            let n = |u, v| check(id.face, 1, u, v);
            let ao_bl = Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1));
            let ao_br = Self::calculate_ao(n(1, 0),  n(0, -1), n(1, -1));
            let ao_tr = Self::calculate_ao(n(1, 0),  n(0, 1),  n(1, 1));
            let ao_tl = Self::calculate_ao(n(-1, 0), n(0, 1),  n(-1, 1));
            Self::quad(verts, inds, idx, [o_bl, o_br, o_tr, o_tl], [apply(ao_bl), apply(ao_br), apply(ao_tr), apply(ao_tl)], true); 
        }

        if !has_btm {
            let c = apply(0.4); 
            Self::quad(verts, inds, idx, [i_tl, i_tr, i_br, i_bl], [c,c,c,c], true); 
        }

        let side_c = apply(0.8); 
        let colors = [side_c, side_c, side_c, side_c];

        if !has_front { Self::quad(verts, inds, idx, [i_bl, i_br, o_br, o_bl], colors, false); }
        if !has_back  { Self::quad(verts, inds, idx, [o_tl, o_tr, i_tr, i_tl], colors, false); }
        if !has_left  { Self::quad(verts, inds, idx, [i_tl, i_bl, o_bl, o_tl], colors, false); }
        if !has_right { Self::quad(verts, inds, idx, [i_br, i_tr, o_tr, o_br], colors, false); }
    }
    pub fn generate_cylinder(radius: f32, height: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = [0.0, 0.5, 1.0]; 

        
        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let z = theta.sin() * radius;
            let normal = Vec3::new(x, 0.0, z).normalize().to_array();

         
            verts.push(Vertex { pos: [x, 0.0, z], color, normal });
            
            verts.push(Vertex { pos: [x, height, z], color, normal });
        }

        for i in 0..segments {
            let bottom1 = i * 2;
            let top1 = bottom1 + 1;
            let bottom2 = bottom1 + 2;
            let top2 = bottom1 + 3;

            inds.push(bottom1); inds.push(top1); inds.push(bottom2);
            inds.push(bottom2); inds.push(top1); inds.push(top2);
        }

        
        let center_idx = verts.len() as u32;
        verts.push(Vertex { pos: [0.0, height, 0.0], color, normal: [0.0, 1.0, 0.0] });
        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let z = theta.sin() * radius;
            verts.push(Vertex { pos: [x, height, z], color, normal: [0.0, 1.0, 0.0] });
        }
        for i in 0..segments {
            inds.push(center_idx);
            inds.push(center_idx + 1 + i);
            inds.push(center_idx + 1 + i + 1);
        }

        (verts, inds)
    }



    
    pub fn generate_sphere_guide(radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = [1.0, 1.0, 1.0]; 

        for y in 0..=segments {
            for x in 0..=segments {
                let x_segment = x as f32 / segments as f32;
                let y_segment = y as f32 / segments as f32;
                let x_pos = (x_segment * std::f32::consts::TAU).cos() * (y_segment * std::f32::consts::PI).sin();
                let y_pos = (y_segment * std::f32::consts::PI).cos();
                let z_pos = (x_segment * std::f32::consts::TAU).sin() * (y_segment * std::f32::consts::PI).sin();

                verts.push(Vertex {
                    pos: [x_pos * radius, y_pos * radius, z_pos * radius],
                    color,
                    normal: [x_pos, y_pos, z_pos],
                });
            }
        }

        for y in 0..segments {
            for x in 0..segments {
                let i = (y * (segments + 1)) + x;
                inds.push(i);
                inds.push(i + segments + 1);
                inds.push(i + segments + 2);
                
                inds.push(i + segments + 2);
                inds.push(i + 1);
                inds.push(i);
            }
        }

        (verts, inds)
    }



// generates a simple 2D crosshair for the center of the screen
    pub fn generate_crosshair() -> (Vec<Vertex>, Vec<u32>) {
        let s = 0.02; // size relative to screen (2%)
        let color = [1.0, 1.0, 1.0]; 
        let normal = [0.0, 0.0, 1.0]; 

        let verts = vec![
           
            Vertex { pos: [-s, 0.0, 0.0], color, normal },
            Vertex { pos: [ s, 0.0, 0.0], color, normal },
            
            Vertex { pos: [0.0, -s, 0.0], color, normal },
            Vertex { pos: [0.0,  s, 0.0], color, normal },
        ];
        let inds = vec![0, 1, 2, 3];
        (verts, inds)
    }





    fn quad(verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, idx: &mut u32, pos: [Vec3; 4], colors: [[f32; 3]; 4], force_radial: bool) {
        let normal = if force_radial {
            let center = (pos[0] + pos[1] + pos[2] + pos[3]) * 0.25;
            center.normalize().to_array()
        } else {
            (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize().to_array()
        };

       
        for i in 0..4 {
            verts.push(Vertex { pos: pos[i].to_array(), color: colors[i], normal });
        }
        
        inds.push(*idx); inds.push(*idx+1); inds.push(*idx+2);
        inds.push(*idx+2); inds.push(*idx+3); inds.push(*idx);
        *idx += 4;
    }
}