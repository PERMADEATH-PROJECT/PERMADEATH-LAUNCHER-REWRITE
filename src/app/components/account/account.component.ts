import { Component, ChangeDetectorRef, NgZone } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { AuthService } from '@app/services/auth.service';
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import { LucideAngularModule, LogIn, User, TriangleAlert, LogOut, ShieldCheck, UserRound } from 'lucide-angular';
import { Observable } from 'rxjs';
import {UserData} from "@app/models/userdata";

@Component({
  selector: 'app-account',
  standalone: true,
  imports: [CommonModule, FormsModule, LucideAngularModule],
  templateUrl: './account.component.html',
  styleUrls: ['./account.component.css']
})
export class AccountComponent {
  readonly LogIn = LogIn;
  readonly User = User;
  readonly TriangleAlert = TriangleAlert;
  readonly LogOut = LogOut;
  readonly ShieldCheck = ShieldCheck;
  readonly UserRound = UserRound;

  loginUsername = '';
  loginPassword = '';

  regUsername = '';
  regPassword = '';
  regConfirm = '';
  regCode = '';
  showRegisterModal = false;

  user$: Observable<any>;

  constructor(
    public auth: AuthService,
    private cdr: ChangeDetectorRef,
    private ngZone: NgZone
  ) {
    this.user$ = this.auth.user$;
  }

  async onLogin() {
    try {
      const success = await invoke<boolean>('login_user', {
        username: this.loginUsername,
        password: this.loginPassword
      });

      if (success) {
        await message('Login successful', { title: 'Success', kind: 'info' });
        const userData = await invoke<UserData>('load_user_data', { username: this.loginUsername });
        if (userData) {
          this.auth.setUser({ ...userData, username: this.loginUsername });
          this.ngZone.run(() => this.cdr.detectChanges());
        }
      } else {
        await message('Invalid credentials', { title: 'Error', kind: 'error' });
      }
    } catch (err: any) {
      await message(String(err), { title: 'Error', kind: 'error' });
    }
  }

  async onRegister() {
    if (this.regPassword !== this.regConfirm) {
      await message('Passwords do not match', { title: 'Error', kind: 'error' });
      return;
    }
    try {
      const msg = await invoke<string>('register_user', {
        username: this.regUsername,
        password: this.regPassword,
        inviteCode: this.regCode
      });
      await message(msg, { title: 'Account Created', kind: 'info' });
      this.ngZone.run(() => this.closeModal());
    } catch (err: any) {
      await message(String(err), { title: 'Registration Error', kind: 'error' });
    }
  }

  async onLogout() {
    await this.auth.logout();
    this.ngZone.run(() => this.cdr.detectChanges());
    await message('You have been logged out successfully.', { title: 'Session Closed', kind: 'info' });
  }

  openModal() { this.showRegisterModal = true; }
  closeModal() { this.showRegisterModal = false; }
}