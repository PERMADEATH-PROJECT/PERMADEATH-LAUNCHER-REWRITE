import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TauriService } from '@app/services/tauri.service';
import { message } from '@tauri-apps/plugin-dialog';
import { LucideAngularModule, Download, RefreshCw, AlertCircle } from 'lucide-angular';

@Component({
    selector: 'app-updates',
    standalone: true,
    imports: [CommonModule, LucideAngularModule],
    templateUrl: './updates.component.html',
    styleUrls: ['./updates.component.css']
})
export class UpdatesComponent {
    readonly Download = Download;
    readonly RefreshCw = RefreshCw;
    readonly AlertCircle = AlertCircle;

    constructor(public tauri: TauriService) {}

    async toggleUpdate() {
        if (this.tauri.options) {
            this.tauri.options.auto_update = !this.tauri.options.auto_update;
            await this.save();
        }
    }

    async toggleNotifications() {
        if (this.tauri.options) {
            this.tauri.options.notification_enabled = !this.tauri.options.notification_enabled;
            await this.save();
        }
    }

    private async save() {
        const status = await this.tauri.saveOptions();
        if (status) await message('Saved successfully', { title: 'Success', kind: 'info' });
    }
}