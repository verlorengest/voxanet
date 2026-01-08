//engine controller

use glam::{Vec3, Mat4, Vec2};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{PhysicalKey, KeyCode};
use crate::common::*;
use crate::gen::CoordSystem;
use crate::entity::Player;
use crate::physics::Physics;

pub struct Controller {
    
    pub cam_dist: f32,
    pub cam_yaw: f32,
    pub cam_pitch: f32,
    
    // input State
    pub mouse_pos: Vec2,
    pub mouse_delta: (f32, f32),
    pub is_orbiting: bool,
    pub is_wireframe: bool,
    pub show_collisions: bool,
    pub fly_mode: bool, 
    pub sprint: bool,
    pub freeze_culling: bool, 
    pub cursor_id: Option<BlockId>,

    
    pub first_person: bool,
    
    
    keys: [bool; 5], // W, A, S, D, Space
}

impl Controller {
    pub fn new() -> Self {
        Self {
            cam_dist: 200.0, 
            cam_yaw: 0.0,
            cam_pitch: 0.5,
            mouse_pos: Vec2::ZERO,
            mouse_delta: (0.0, 0.0),
            is_orbiting: false,
            cursor_id: None, 
            is_wireframe: false,
            show_collisions: false,
            fly_mode: false,
            freeze_culling: false,
            sprint: false,
            first_person: true,
            keys: [false; 5],
        }
    }

    pub fn update_player(&mut self, player: &mut Player, planet: &PlanetData, dt: f32) {
        

        // read inputs regardless of the view mode.
       
        
        let mut input = Vec3::ZERO;
        if self.keys[0] { input.z -= 1.0; } // W
        if self.keys[1] { input.x -= 1.0; } // A
        if self.keys[2] { input.z += 1.0; } // S
        if self.keys[3] { input.x += 1.0; } // D
        let jump = self.keys[4]; // space

        let rotation_delta = if self.first_person { self.mouse_delta } else { (0.0, 0.0) };

        

        player.update(dt, planet, input, jump, rotation_delta, self.fly_mode, self.sprint);

        
        // reset delta after use
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn get_camera_pos(&self, player: &Player) -> Vec3 {
        if self.first_person {
            // first person: Camera is at player position + eye height
            player.position + (Physics::get_up_vector(player.position) * 1.6)
        } else {
            
            let up = Physics::get_up_vector(player.position);
            player.position + (up * self.cam_dist)
        }
    }


    pub fn process_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.first_person {
            // accumulate raw mouse delta
            self.mouse_delta.0 += delta.0 as f32;
            self.mouse_delta.1 += delta.1 as f32;
        }
    }


    pub fn process_events(&mut self, event: &WindowEvent, _player: &mut Player, _planet: &PlanetData) -> bool {

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                let d = new_pos - self.mouse_pos;
                self.mouse_pos = new_pos;
                self.mouse_delta = (d.x, d.y);                
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Middle {
                    self.is_orbiting = *state == ElementState::Pressed;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if !self.first_person {
                    let y = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                    };
                    
                    self.cam_dist = (self.cam_dist - y * 50.0).clamp(10.0, 10000.0);
                    return true;
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.keys[0] = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.keys[1] = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.keys[2] = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.keys[3] = pressed,
                    PhysicalKey::Code(KeyCode::Space) => self.keys[4] = pressed,
                   
                    PhysicalKey::Code(KeyCode::ControlLeft) => self.sprint = pressed, 
                    
                    PhysicalKey::Code(KeyCode::KeyP) if pressed => { 
                      
                        if _player.debug_mode {
                            self.is_wireframe = !self.is_wireframe; 
                        }
                        return true; 
                    }
                   
                    PhysicalKey::Code(KeyCode::KeyO) if pressed => {
                        if _player.debug_mode {
                            self.show_collisions = !self.show_collisions;
                            println!("Show Collisions: {}", self.show_collisions);
                        }
                        return true;
                    }

                    PhysicalKey::Code(KeyCode::Quote) if pressed => {
                        if _player.debug_mode {
                            self.freeze_culling = !self.freeze_culling;
                        }
                        return true;
                    }
                    
                    PhysicalKey::Code(KeyCode::KeyK) if pressed => {
                        self.first_person = !self.first_person;
                     
                        
                        if self.first_person { self.cam_dist = 40.0; } 
                        else { self.cam_dist = 100.0; }
                        return true;
                    }
                    
                    PhysicalKey::Code(KeyCode::KeyF) if pressed => {
                        if self.first_person {
                            self.fly_mode = !self.fly_mode;
                            println!("Fly Mode: {}", self.fly_mode);
                        }
                        return true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }

pub fn get_matrix(&self, player: &Player, width: f32, height: f32) -> Mat4 {

        // use 45 degrees in Orbit mode for less distortion.
        let fov_degrees: f32 = if self.first_person { 80.0 } else { 45.0 };

        // far plane increased to 20,000 for massive zoom out
        let proj = Mat4::perspective_rh(fov_degrees.to_radians(), width / height, 0.1, 20000.0);
        
        let view = if self.first_person {
            player.get_view_matrix()
        } else {
          
            let up = Physics::get_up_vector(player.position);
            let cam_pos = player.position + (up * self.cam_dist);
            let target = player.position;
            
         
            let player_forward = player.rotation * Vec3::NEG_Z;
            
            Mat4::look_at_rh(cam_pos, target, player_forward)
        };
        
        proj * view
    }

pub fn raycast(&self, player: &Player, planet: &PlanetData, width: f32, height: f32, place_mode: bool) -> Option<(BlockId, f32)> {
        let mvp = self.get_matrix(player, width, height);
        let inv = mvp.inverse();
        
        let (ndc_x, ndc_y) = if self.first_person {
            (0.0, 0.0) 
        } else {
            ((2.0 * self.mouse_pos.x / width) - 1.0, 1.0 - (2.0 * self.mouse_pos.y / height))
        };

        let start = inv.project_point3(Vec3::new(ndc_x, ndc_y, 0.0));
        let end = inv.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));
        let dir = (end - start).normalize();

        let mut dist = 0.0;
        let mut last_empty = None;
        
       
        let reach = if self.first_person { 8.0 } else { self.cam_dist + 100.0 };
        // stop raycast if we hit the absolute math center (radius < 0.5)
        let min_radius = 0.5;

        while dist < reach {
            let p = start + dir * dist;
            if p.length() < min_radius { break; }
            
       
            // since blocks are now approx 1.0 unit thick/wide, 0.25 is a safe step.
            let step = 0.25;

            if let Some(id) = CoordSystem::pos_to_id(p, planet.resolution) {
                let exists = planet.exists(id);
                if place_mode {
                    if exists { return last_empty.map(|i| (i, dist)); }
                    else { last_empty = Some(id); }
                } else {
                    if exists { return Some((id, dist)); }
                }
            }
            dist += step;
        }
        None
    }
}