//lighting.rs

use crate::common::*;
use crate::gen::CoordSystem;
use std::collections::{VecDeque, HashSet};

pub struct LightEngine;

impl LightEngine {
    const MAX_LIGHT: u8 = 15;
    const SUNLIGHT_START: u8 = 15;
    
    pub fn calculate_light(id: BlockId, planet: &mut PlanetData) -> u8 {
        if let Some(&cached) = planet.light_cache.get(&id) {
            return cached;
        }
        
        let light = Self::trace_sunlight(id, planet);
        planet.light_cache.insert(id, light);
        light
    }
    
    fn trace_sunlight(id: BlockId, planet: &PlanetData) -> u8 {
        let res = planet.resolution;
        let mut current_light = Self::SUNLIGHT_START;
        
        for i in 1..=8 {
            let check_layer = id.layer as i32 + i;
            if check_layer >= res as i32 {
                break;
            }
            
            let check_id = BlockId {
                face: id.face,
                layer: check_layer as u32,
                u: id.u,
                v: id.v,
            };
            
            if planet.exists(check_id) {
                current_light = current_light.saturating_sub(8);
                if current_light == 0 {
                    return 0;
                }
            }
        }
        
        current_light
    }
    
    pub fn propagate_area(center: BlockId, planet: &mut PlanetData, radius: u32) {
        let res = planet.resolution;
        
        for du in -(radius as i32)..=(radius as i32) {
            for dv in -(radius as i32)..=(radius as i32) {
                for dl in -(radius as i32)..=(radius as i32) {
                    let u = (center.u as i32 + du).clamp(0, res as i32 - 1) as u32;
                    let v = (center.v as i32 + dv).clamp(0, res as i32 - 1) as u32;
                    let l = (center.layer as i32 + dl).clamp(0, res as i32 - 1) as u32;
                    
                    let id = BlockId { face: center.face, layer: l, u, v };
                    planet.light_cache.remove(&id);
                }
            }
        }
    }
}