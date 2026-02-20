import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink, RouterLinkActive } from '@angular/router';
import {
  LucideAngularModule, Skull, Swords, Settings, Zap,
  Download, LogIn, LogOut, Heart
} from 'lucide-angular';
import { exit } from '@tauri-apps/plugin-process';

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive, LucideAngularModule],
  templateUrl: './sidebar.component.html',
  styleUrls: ['./sidebar.component.css']
})
export class SidebarComponent {
  readonly Skull = Skull;
  readonly Heart = Heart;
  readonly Swords = Swords;
  readonly Settings = Settings;
  readonly Zap = Zap;
  readonly Download = Download;
  readonly LogIn = LogIn;
  readonly LogOut = LogOut;

  async exitApp() {
    await exit(0);
  }
}