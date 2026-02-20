import { Component, OnInit, NgZone } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TauriService } from '@app/services/tauri.service';
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import { LucideAngularModule, Zap, Info, MemoryStick, Gauge, TriangleAlert } from 'lucide-angular';
import { JavaVersions } from '@app/models/java-versions.enum';

@Component({
  selector: 'app-vm',
  standalone: true,
  imports: [CommonModule, FormsModule, LucideAngularModule],
  templateUrl: './vm.component.html',
  styleUrls: ['./vm.component.css']
})
export class VmComponent implements OnInit {
  readonly Zap = Zap;
  readonly Info = Info;
  readonly MemoryStick = MemoryStick;
  readonly Gauge = Gauge;
  readonly TriangleAlert = TriangleAlert;

  gcOptions: string[] = [];
  javaVersions = Object.values(JavaVersions);

  maxRamStr = '4096MB';
  jvmArgsStr = '';

  constructor(public tauri: TauriService, private ngZone: NgZone) {}

  async ngOnInit() {
    this.gcOptions = await invoke<string[]>('get_garbage_collectors');
    if (this.tauri.gameOptions) {
      this.maxRamStr = `${this.tauri.gameOptions.max_ram}MB`;
      this.jvmArgsStr = this.tauri.gameOptions.vm_flags.join(' ');
    }
    this.ngZone.run(() => {});
  }

  /** Persists all JVM options including GC and Java version */
  async saveVmOptions() {
    if (!this.tauri.gameOptions) return;

    const maxRam = parseInt(this.maxRamStr.replace('MB', ''));
    if (isNaN(maxRam) || maxRam <= 0) {
      await message('Please enter a valid positive number for maximum RAM.', { title: 'Invalid Input', kind: 'error' });
      return;
    }

    this.tauri.gameOptions.max_ram = maxRam;
    this.tauri.gameOptions.vm_flags = this.jvmArgsStr.split(' ').filter(f => f.trim() !== '');

    const status = await this.tauri.saveGameOptions();
    if (status) {
      await message('Game options saved successfully', { title: 'Save Game Options', kind: 'info' });
    } else {
      await message('Failed to save game options.', { title: 'Save Game Options', kind: 'error' });
    }
  }

  async resetDefaults() {
    const defaultFlags = await invoke<string[]>('get_base_jvm_flags');
    this.jvmArgsStr = defaultFlags.join(' ');
    if (this.tauri.gameOptions) {
      this.tauri.gameOptions.vm_flags = defaultFlags;
      const status = await this.tauri.saveGameOptions();
      if (status) {
        await message('Game options reset successfully', { title: 'Reset Game Options', kind: 'info' });
      } else {
        await message('Failed to reset game options.', { title: 'Reset Game Options', kind: 'error' });
      }
    }
  }
}