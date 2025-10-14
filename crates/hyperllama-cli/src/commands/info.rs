use anyhow::Result;
use hyperllama_core::Hardware;

pub async fn execute() -> Result<()> {
    let hw = Hardware::detect();

    println!("System Information:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Hardware Type: {:?}", hw.hw_type);
    println!("CPU Cores: {}", hw.cpu_cores);
    println!("RAM Total: {} MB", hw.ram_total_mb);
    println!("RAM Available: {} MB", hw.ram_available_mb);

    if let Some(gpu) = hw.gpu_info {
        println!("\nGPU Information:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("GPU Name: {}", gpu.name);
        println!("VRAM Total: {} MB", gpu.vram_total_mb);
        println!("VRAM Available: {} MB", gpu.vram_available_mb);
        if let Some((major, minor)) = gpu.compute_capability {
            println!("Compute Capability: {}.{}", major, minor);
        }
    }

    Ok(())
}
