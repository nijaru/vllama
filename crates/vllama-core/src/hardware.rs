use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareType {
    Cpu,
    NvidiaGpu,
    AmdGpu,
    AppleSilicon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_total_mb: u64,
    pub vram_available_mb: u64,
    pub compute_capability: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub hw_type: HardwareType,
    pub cpu_cores: usize,
    pub ram_total_mb: u64,
    pub ram_available_mb: u64,
    pub gpu_info: Option<GpuInfo>,
}

impl Hardware {
    pub fn detect() -> Self {
        let sys = sysinfo::System::new_all();

        let cpu_cores = sys.cpus().len();
        let ram_total_mb = sys.total_memory() / (1024 * 1024);
        let ram_available_mb = sys.available_memory() / (1024 * 1024);

        #[cfg(target_os = "macos")]
        let (hw_type, gpu_info) = {
            if std::env::consts::ARCH == "aarch64" {
                (HardwareType::AppleSilicon, None)
            } else {
                (HardwareType::Cpu, None)
            }
        };

        #[cfg(not(target_os = "macos"))]
        let (hw_type, gpu_info) = (HardwareType::Cpu, None);

        Self {
            hw_type,
            cpu_cores,
            ram_total_mb,
            ram_available_mb,
            gpu_info,
        }
    }

    pub fn has_gpu(&self) -> bool {
        matches!(
            self.hw_type,
            HardwareType::NvidiaGpu | HardwareType::AmdGpu | HardwareType::AppleSilicon
        )
    }

    pub fn available_vram_mb(&self) -> Option<u64> {
        self.gpu_info.as_ref().map(|gpu| gpu.vram_available_mb)
    }
}
