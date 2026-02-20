import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { exit } from '@tauri-apps/plugin-process';
import {GameOptions, LauncherOptions} from "@app/models/options";

@Injectable({ providedIn: 'root' })
export class TauriService {
    options: LauncherOptions | null = null;
    gameOptions: GameOptions | null = null;

    async initOptions() {
        try {
            this.options = await invoke<LauncherOptions>('read_options');
            if (!this.options) {
                await exit(0);
                return;
            }
            this.gameOptions = await invoke<GameOptions>('read_game_options', { launcherOptions: this.options });
        } catch (error) {
            console.error("Error loading options:", error);
            await exit(1);
        }
    }

    async saveOptions() {
        return await invoke('save_options', { options: this.options });
    }

    async saveGameOptions() {
        return await invoke('save_game_options', { gameOptions: this.gameOptions, launcherOptions: this.options });
    }
}