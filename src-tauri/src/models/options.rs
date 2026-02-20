use std::path::PathBuf;
use dirs_next::config_dir;
use log::info;

pub const BASE_VM_FLAGS: [&str; 10] = [
    "-XX:MaxGCPauseMillis=100",
    "-XX:G1NewSizePercent=30",
    "-XX:G1ReservePercent=20",
    "-XX:+UseStringDeduplication",
    "-XX:G1HeapRegionSize=32M",
    "-XX:+TieredCompilation",
    "-XX:+AlwaysPreTouch",
    "-Dsun.java2d.opengl=true",
    "-Xverify:none",
    "-XX:+UnlockExperimentalVMOptions"
];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum GarbageCollector {
    Serial,
    Parallel,
    G1GC,
    ZGC,
    Shenandoah,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LauncherOptions {
    pub launcher_dir: Option<PathBuf>,
    pub game_dir: Option<PathBuf>,
    pub init_on_start: bool,
    pub auto_update: bool,
    pub notification_enabled: bool,
    pub debug_console: bool,
    automatic_backup: bool,
}

impl LauncherOptions {
    pub fn new() -> Self {
        let launcher_dir = config_dir().map(|mut path| {
            path.push(".Permadeath-Launcher");
            info!("Launcher directory configured at: {:?}", path);
            path
        });

        let game_dir = dirs_next::data_dir().map(|mut path| {
            path.push(".Permadeath");
            info!("Game directory configured at: {:?}", path);
            path
        });

        Self {
            launcher_dir,
            game_dir,
            init_on_start: false,
            auto_update: true,
            notification_enabled: false,
            debug_console: false,
            automatic_backup: true,
        }
    }

    pub fn get_default_game_dir() -> Option<PathBuf> {
        dirs_next::data_dir().map(|mut path| {
            path.push(".Permadeath");
            path
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameOptions {
    max_ram: u32,
    vm_flags: Vec<String>,
    garbage_collector: GarbageCollector,
}

impl GameOptions {
    pub fn new() -> Self {
        Self {
            max_ram: 4096,
            vm_flags: BASE_VM_FLAGS.iter().map(|s| s.to_string()).collect(),
            garbage_collector: GarbageCollector::G1GC,
        }
    }

    pub fn get_max_ram(&self) -> u32 {
        self.max_ram
    }

    pub fn get_vm_flags(&self) -> Vec<String> {
        let mut flags = self.vm_flags.clone();
        let gc_flag = match self.garbage_collector {
            GarbageCollector::Serial => "-XX:+UseSerialGC",
            GarbageCollector::Parallel => "-XX:+UseParallelGC",
            GarbageCollector::G1GC => "-XX:+UseG1GC",
            GarbageCollector::ZGC => "-XX:+UseZGC",
            GarbageCollector::Shenandoah => "-XX:+UseShenandoahGC",
        };
        flags.push(gc_flag.into());
        flags
    }

    pub fn set_max_ram(&mut self, ram_mb: u32) {
        self.max_ram = ram_mb;
    }

    pub fn add_vm_flag(&mut self, flag: String) {
        if !self.vm_flags.contains(&flag) {
            self.vm_flags.push(flag);
        }
    }

    pub fn remove_vm_flag(&mut self, flag: &str) {
        self.vm_flags.retain(|f| f != flag);
    }

    pub fn set_garbage_collector(&mut self, gc: GarbageCollector) {
        self.garbage_collector = gc;
    }

    pub fn get_garbage_collectors() -> Vec<GarbageCollector> {
        vec![
            GarbageCollector::Serial,
            GarbageCollector::Parallel,
            GarbageCollector::G1GC,
            GarbageCollector::ZGC,
            GarbageCollector::Shenandoah,
        ]
    }
}
