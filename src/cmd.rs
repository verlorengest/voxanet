use crate::entity::Player;

pub struct Console {
    pub is_open: bool,
    pub input_buffer: String,
    pub history: Vec<(String, [f32; 3])>, 
    pub height_fraction: f32, 
    
   
    history_capacity: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            is_open: false,
            input_buffer: String::new(),
            history: Vec::new(),
            height_fraction: 0.0,
            history_capacity: 50,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            
            self.input_buffer.clear();
        }
    }

    pub fn log(&mut self, text: &str, color: [f32; 3]) {
        // print to actual terminal
        println!("{}", text);
        
        if self.history.len() >= self.history_capacity {
            self.history.remove(0);
        }
        self.history.push((text.to_string(), color));
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.is_open { return; }
        // filter control characters
        if !c.is_control() {
            self.input_buffer.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if !self.is_open { return; }
        self.input_buffer.pop();
    }

    pub fn submit(&mut self, player: &mut Player) {
        if self.input_buffer.is_empty() { return; }
        
        let cmd = self.input_buffer.clone();
        self.log(&format!("> {}", cmd), [1.0, 1.0, 1.0]); // log
        
        self.process_command(&cmd, player);
        self.input_buffer.clear();
    }

    fn process_command(&mut self, cmd_line: &str, player: &mut Player) {
        let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
        if parts.is_empty() { return; }

        let command = parts[0];

        match command {
            "/move_speed" => {
                self.handle_property_command(parts, "move_speed", &mut player.move_speed);
            },
            "/jump_force" => {
                self.handle_property_command(parts, "jump_force", &mut player.jump_force);
            },
            
            "/debug_mode" => {
                 if parts.len() < 3 || parts[1] != "set" {
                    self.log("Usage: /debug_mode set [true/false]", [1.0, 0.5, 0.0]);
                    return;
                }
                match parts[2] {
                    "true" => { player.debug_mode = true; self.log("Debug Mode: ON", [0.0, 1.0, 0.0]); },
                    "false" => { player.debug_mode = false; self.log("Debug Mode: OFF", [1.0, 0.0, 0.0]); },
                    _ => self.log("Value must be true or false", [1.0, 0.0, 0.0]),
                }
            },
         
            "help" => {
                self.log("Available Commands:", [0.0, 1.0, 1.0]);
                self.log("  /debug_mode set true", [0.8, 0.8, 0.8]); 
                self.log("  /move_speed set {value}", [0.8, 0.8, 0.8]);
                self.log("  /jump_force set {value}", [0.8, 0.8, 0.8]);
            },
            _ => {
                self.log(&format!("Unknown command: {}", command), [1.0, 0.0, 0.0]);
            }
        }
    }

    fn handle_property_command(&mut self, parts: Vec<&str>, name: &str, property: &mut f32) {
        if parts.len() < 2 {
            self.log(&format!("Usage: /{} [set/get]", name), [1.0, 0.5, 0.0]);
            return;
        }

        match parts[1] {
            "get" => {
                self.log(&format!("{} is currently: {:.2}", name, property), [0.0, 1.0, 0.0]);
            },
            "set" => {
                if parts.len() < 3 {
                    self.log(&format!("Usage: /{} set <value>", name), [1.0, 0.5, 0.0]);
                    return;
                }
                match parts[2].parse::<f32>() {
                    Ok(val) => {
                        *property = val;
                        self.log(&format!("{} set to {:.2}", name, val), [0.0, 1.0, 0.0]);
                    },
                    Err(_) => {
                        self.log("Invalid number format.", [1.0, 0.0, 0.0]);
                    }
                }
            },
            _ => {
                self.log(&format!("Unknown operation '{}'. Use set or get.", parts[1]), [1.0, 0.5, 0.0]);
            }
        }
    }

    pub fn update_animation(&mut self, dt: f32) {
        let speed = 5.0;
        if self.is_open {
            self.height_fraction = (self.height_fraction + dt * speed).min(1.0);
        } else {
            self.height_fraction = (self.height_fraction - dt * speed).max(0.0);
        }
    }
}