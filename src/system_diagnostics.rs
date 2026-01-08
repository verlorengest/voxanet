use sysinfo::System;

pub struct SystemDiagnostics;

impl SystemDiagnostics {
    pub fn print_startup_info() {
        let mut sys = System::new_all();
        sys.refresh_all();

        println!("\n==========================================");
        println!("           SYSTEM DIAGNOSTICS       ");
        println!("==========================================");
        
        // OS
        let os_name = System::name().unwrap_or("Unknown".to_string());
        let os_ver = System::os_version().unwrap_or("".to_string());
        println!("OS       : {} {}", os_name, os_ver);
        println!("Kernel   : {}", System::kernel_version().unwrap_or("Unknown".to_string()));
        println!("Hostname : {}", System::host_name().unwrap_or("Unknown".to_string()));

        // CPU
        let cpus = sys.cpus();
        if !cpus.is_empty() {
            println!("CPU      : {} ", cpus[0].brand().trim());
            println!("Cores    : {} Logical Cores", cpus.len());
        }

        // RAM
        let total_ram = sys.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        let used_ram = sys.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        println!("Memory   : {:.2} GB used / {:.2} GB total", used_ram, total_ram);
        
        println!("==========================================\n");
    }

    pub fn log_gpu(info: &wgpu::AdapterInfo) {
        println!("--- GPU INFO ---");
        println!("Name     : {}", info.name);
        println!("Backend  : {:?}", info.backend);
        println!("Driver   : {}", info.driver);
        println!("Vendor   : {:?}", info.vendor);
        println!("----------------\n");
    }
}