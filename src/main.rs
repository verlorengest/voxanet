// engine main.rs

mod common;
mod gen;
mod physics;
mod entity;
mod controller;
mod renderer;
mod noise;
mod lod_animation;
mod cmd;
mod system_diagnostics; 



use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent}; // Added DeviceEvent
use winit::event_loop::EventLoop;
use winit::window::{WindowBuilder, CursorGrabMode};
use winit::keyboard::{Key, PhysicalKey, KeyCode};
use crate::common::PlanetData;
use crate::renderer::Renderer;
use crate::controller::Controller;
use crate::entity::Player;
use crate::cmd::Console;
use crate::system_diagnostics::SystemDiagnostics;
use std::time::Instant;



fn main() {
    
    SystemDiagnostics::print_startup_info(); 
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("voxanet").build(&event_loop).unwrap();
    
    let mut renderer = pollster::block_on(Renderer::new(&window));
    let mut controller = Controller::new();
    let mut player = Player::new();
    let mut planet = PlanetData::new(49); // Keep high resolution

    let mut console = Console::new();
    console.log("Welcome to voxanet.", [0.0, 1.0, 0.0]);
    console.log("Press ` to open console.", [1.0, 1.0, 1.0]);


    // initialize player spawn

    // we query the height at face 0, u=res/2, v=res/2 (roughly the "North Pole" of face 0)
    let center = planet.resolution / 2;
    let ground_level = planet.terrain.get_height(0, center, center);
    let spawn_h = crate::gen::CoordSystem::get_layer_radius(ground_level, planet.resolution) + 10.0;
   

    player.spawn(glam::Vec3::new(0.0, spawn_h, 0.0));
    let mut last_time = Instant::now();
    let mut current_mode_first_person = false; 

    event_loop.run(move |event, target| {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        // cursor locking logic 
        if controller.first_person != current_mode_first_person {
            current_mode_first_person = controller.first_person;
            if current_mode_first_person {
                let _ = renderer.window.set_cursor_grab(CursorGrabMode::Locked);
                renderer.window.set_cursor_visible(false);
            } else {
                let _ = renderer.window.set_cursor_grab(CursorGrabMode::None);
                renderer.window.set_cursor_visible(true);
            }
        }
        
        // physics & player Update
        controller.update_player(&mut player, &planet, dt);
        
        // raycast & cursor Update
        let width = renderer.config.width as f32;
        let height = renderer.config.height as f32;
        let ray_result = controller.raycast(&player, &planet, width, height, false);
        controller.cursor_id = ray_result.map(|(id, _)| id);
        
        renderer.update_cursor(&planet, controller.cursor_id);
        renderer.update_view(player.position, &planet);


        // UPDATE ANIMATION
        console.update_animation(dt);

        // BLOCK CONTROLS IF CONSOLE OPEN
        // Only update player/physics if console is NOT hijacking input
        if !console.is_open {
             // (Existing Physics & Player Update)
             controller.update_player(&mut player, &planet, dt);
             
            
             let width = renderer.config.width as f32;
             let height = renderer.config.height as f32;
             let ray_result = controller.raycast(&player, &planet, width, height, false);
             controller.cursor_id = ray_result.map(|(id, _)| id);
        } else {
            
             let _ = renderer.window.set_cursor_grab(CursorGrabMode::None);
             renderer.window.set_cursor_visible(true);
        }




        match event {
            
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                controller.process_mouse_motion(delta);
            },

            Event::WindowEvent { event, window_id } if window_id == renderer.window.id() => {
                
                
                // CONSOLE INPUT INTERCEPTION
                if console.is_open {
                    match event {
                        WindowEvent::KeyboardInput { event: key_event, .. } => {
                             if key_event.state == ElementState::Pressed {
                                 match key_event.physical_key {
                                     PhysicalKey::Code(KeyCode::Backquote) => console.toggle(),
                                     PhysicalKey::Code(KeyCode::Enter) => console.submit(&mut player),
                                     PhysicalKey::Code(KeyCode::Backspace) => console.handle_backspace(),
                                     _ => {
                                         if let Some(txt) = &key_event.text {
                                             // Append text to console buffer
                                             for c in txt.chars() { console.handle_char(c); }
                                         }
                                     }
                                 }
                             }                            
                             return; 
                        },
                         _ => {} 
                    }
                }
                
                if let WindowEvent::KeyboardInput { event: key_event, .. } = &event {
                     if key_event.state == ElementState::Pressed {
                         if let PhysicalKey::Code(KeyCode::Backquote) = key_event.physical_key {
                             console.toggle();
                             return;
                         }
                     }
                }
                
                
                
                controller.process_events(&event, &mut player, &planet);
                
                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
                    
                    WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                        let is_right = button == MouseButton::Right;
                        if let Some(id) = controller.cursor_id {
                             if is_right { 
                                 let place_info = controller.raycast(&player, &planet, renderer.config.width as f32, renderer.config.height as f32, true);
                                 if let Some((place_id, _)) = place_info {
                                     planet.add_block(place_id);
                                     renderer.refresh_neighbors(place_id, &planet);
                                 }
                             } else { 
                                 planet.remove_block(id); 
                                 renderer.refresh_neighbors(id, &planet);
                             }
                            renderer.window.request_redraw();
                        } else {
                            if controller.first_person {
                                let _ = renderer.window.set_cursor_grab(CursorGrabMode::Locked);
                                renderer.window.set_cursor_visible(false);
                            }
                        }
                    },
                    
                    WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                         if let Key::Character(ref s) = event.logical_key {
                            if s == "]" || s == "[" {
                                if s == "]" { planet.resize(true); } 
                                else { planet.resize(false); }
                                
                                let new_res = planet.resolution;
                                let current_dir = if player.position.length() > 0.1 { player.position.normalize() } else { glam::Vec3::Y };
                                let probe_dist = new_res as f32 / 2.0;
                                let dummy_pos = current_dir * probe_dist; 
                                
                                let spawn_radius = if let Some(id) = crate::gen::CoordSystem::pos_to_id(dummy_pos, new_res) {
                                    let h = planet.terrain.get_height(id.face, id.u, id.v);
                                    crate::gen::CoordSystem::get_layer_radius(h, new_res) + 5.0
                                } else {
                                    (new_res as f32 / 2.0) + 20.0 
                                };

                                player.position = current_dir * spawn_radius;
                                player.velocity = glam::Vec3::ZERO;
                                
                                renderer.force_reload_all(&planet, player.position);
                                renderer.log_memory(&planet);
                                renderer.window.request_redraw();
                            }
                        }
                    },

                    WindowEvent::RedrawRequested => {
                            renderer.render(&controller, &player, &planet, &console);

                        },
                    _ => {}
                }
            },
            Event::AboutToWait => renderer.window.request_redraw(),
            _ => {}
        }
    }).unwrap();
}