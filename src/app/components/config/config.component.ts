import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TauriService } from '../../services/tauri.service';
import { message, open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { LucideAngularModule, Settings, Gamepad2 } from 'lucide-angular';

@Component({
  selector: 'app-config',
  standalone: true,
  imports: [CommonModule, FormsModule, LucideAngularModule],
  templateUrl: './config.component.html',
  styleUrls: ['./config.component.css']
})
export class ConfigComponent {
  readonly Settings = Settings;
  readonly Gamepad2 = Gamepad2;

  constructor(public tauri: TauriService) {}

  async changeGameDir() {
    const gameDir = await open({ multiple: false, directory: true });
    if (gameDir && this.tauri.options) {
      this.tauri.options.game_dir = gameDir as string;
    }
  }

  async saveConfig() {
    const status = await this.tauri.saveOptions();
    if (status) {
      await message('Options saved successfully', { title: 'Save Options', kind: 'info' });
    } else {
      await message('Failed to save options.', { title: 'Save Options', kind: 'error' });
    }
  }

  async resetConfig() {
    const gameDir = await invoke<string>('return_default_game_dir');
    if (this.tauri.options) {
      this.tauri.options.init_on_start = false;
      this.tauri.options.debug_console = false;
      this.tauri.options.automatic_backup = true;
      this.tauri.options.game_dir = gameDir;
      const status = await this.tauri.saveOptions();
      if (status) {
        await message('Options reset successfully', { title: 'Reset Options', kind: 'info' });
      } else {
        await message('Failed to reset options.', { title: 'Reset Options', kind: 'error' });
      }
    }
  }
}