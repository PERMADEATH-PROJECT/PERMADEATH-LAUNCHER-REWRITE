export interface LauncherOptions {
    launcher_dir?: string;
    game_dir?: string;
    init_on_start: boolean;
    auto_update: boolean;
    notification_enabled: boolean;
    debug_console: boolean;
    automatic_backup: boolean;
}

export interface GameOptions {
    max_ram: number;
    vm_flags: string[];
    garbage_collector: string;
    custom_java_path: string;
}
