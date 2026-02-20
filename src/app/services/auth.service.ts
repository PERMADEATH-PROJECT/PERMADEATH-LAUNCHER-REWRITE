import { Injectable, NgZone } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { BehaviorSubject } from 'rxjs';
import { UserData } from '@app/models/userdata';

/** Gestiona el estado de autenticación y sesión del usuario */
@Injectable({ providedIn: 'root' })
export class AuthService {
  private userSubject = new BehaviorSubject<UserData | null>(null);
  user$ = this.userSubject.asObservable();

  constructor(private ngZone: NgZone) {}

  async checkAutoLogin(): Promise<boolean> {
    const session = await invoke<{ user_id: number; username: string } | null>('check_session');
    if (session) {
      const userData = await invoke<any>('load_user_data', { username: session.username });
      if (userData && userData.status !== undefined) {
        this.setUser({ ...userData, username: session.username });
        return true;
      }
    }
    return false;
  }

  /** Actualiza el estado dentro de la zona de Angular para garantizar el refresco de la vista */
  setUser(user: UserData | null) {
    this.ngZone.run(() => this.userSubject.next(user));
  }

  async logout() {
    await invoke('logout');
    this.setUser(null);
  }
}