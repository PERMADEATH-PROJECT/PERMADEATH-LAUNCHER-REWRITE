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
      <!-- Console window: full-screen, no sidebar/footer -->
      <router-outlet *ngIf="isConsoleWindow"></router-outlet>

      <!-- Main launcher window -->
      <div class="main-layout" *ngIf="!isConsoleWindow">
        <app-sidebar></app-sidebar>
        <main class="dashboard">
          <div class="dashboard-center-wrapper">
            <router-outlet></router-outlet>
          </div>
          <app-footer></app-footer>
        </main>
      </div>
    </ng-container>
  `,
  styles: [`
    :host { display: block; height: 100vh; }
    .main-layout { display: flex; min-height: 100vh; width: 100vw; overflow-x: auto; }
    .dashboard { flex: 1; min-width: 0; display: flex; flex-direction: column;
      align-items: center; justify-content: flex-start; padding: 0; width: 100%; overflow-y: auto; }
    .dashboard-center-wrapper { width: 100%; margin: 0 auto; display: flex; flex-direction: column;
      justify-content: flex-start; gap: 16px; padding-top: 24px; flex: 1; }
  `]
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
