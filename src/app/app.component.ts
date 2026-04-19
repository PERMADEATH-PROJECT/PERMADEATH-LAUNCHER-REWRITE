import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterOutlet } from '@angular/router';
import { TauriService } from './services/tauri.service';
import { AuthService } from './services/auth.service';
import { SidebarComponent } from '@app/components/sidebar/sidebar.component';
import { FooterComponent } from '@app/components/footer/footer.component';
import { getCurrentWindow } from '@tauri-apps/api/window';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SidebarComponent, FooterComponent],
  template: `
    <ng-container *ngIf="isLoaded">
      <ng-container *ngIf="isConsoleWindow">
        <router-outlet></router-outlet>
      </ng-container>

      <div class="flex h-screen w-screen overflow-hidden" *ngIf="!isConsoleWindow">
        <app-sidebar></app-sidebar>
        <main class="flex-1 min-w-0 flex flex-col overflow-y-auto">
          <div class="w-full flex flex-col gap-4 pt-6 flex-1 px-6 max-w-screen-xl mx-auto">
            <router-outlet></router-outlet>
          </div>
          <app-footer></app-footer>
        </main>
      </div>
    </ng-container>
  `,
  styles: [`:host { display: block; height: 100vh; }`]
})
export class AppComponent implements OnInit {
  isLoaded = false;
  isConsoleWindow = false;

  constructor(
    private tauri: TauriService,
    private auth: AuthService,
    private router: Router,
  ) {}

  async ngOnInit(): Promise<void> {
    const winLabel = (await getCurrentWindow()).label;
    this.isConsoleWindow = winLabel === 'console';

    if (this.isConsoleWindow) {
      // Navigate to the console route and skip launcher init
      await this.router.navigate(['/console']);
    } else {
      await this.tauri.initOptions();
      await this.auth.checkAutoLogin();
    }

    this.isLoaded = true;
  }
}
