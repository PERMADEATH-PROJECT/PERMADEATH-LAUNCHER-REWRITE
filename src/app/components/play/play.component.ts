import {
  Component, OnInit, OnDestroy, NgZone, ViewChild, ElementRef, AfterViewChecked
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { Subscription } from 'rxjs';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { message } from '@tauri-apps/plugin-dialog';
import {
  LucideAngularModule,
  Skull, Play, Users, Square, Terminal, Lock, Download, Signal, SignalLow, Globe
} from 'lucide-angular';
import { AuthService } from '@app/services/auth.service';
import { TauriService } from '@app/services/tauri.service';
import { UserData } from '@app/models/userdata';

export type GameState = 'idle' | 'downloading' | 'running';

interface LogLine { text: string; isError: boolean; }

interface ServerStatus {
  online: boolean;
  players_online: number;
  players_max: number;
  player_names: string[];
  version: string;
  motd: string;
}

@Component({
  selector: 'app-play',
  standalone: true,
  imports: [CommonModule, LucideAngularModule],
  templateUrl: './play.component.html',
  styleUrls: ['./play.component.css']
})
export class PlayComponent implements OnInit, OnDestroy, AfterViewChecked {
  readonly Skull    = Skull;
  readonly Play     = Play;
  readonly Users    = Users;
  readonly Square   = Square;
  readonly Terminal = Terminal;
  readonly Lock     = Lock;
  readonly Download = Download;
  readonly Signal    = Signal;
  readonly SignalLow = SignalLow;
  readonly Globe     = Globe;

  gameState: GameState = 'idle';
  logLines: LogLine[]  = [];
  currentUser: UserData | null = null;
  serverStatus: ServerStatus | null = null;
  serverLoading = false;

  // Auth modal
  showAuthModal = false;
  private authModalResolve: ((v: boolean | null) => void) | null = null;

  private shouldScrollLog = false;
  private statusInterval?: ReturnType<typeof setInterval>;
  private userSub?: Subscription;
  private unlistenState?:   UnlistenFn;
  private unlistenLog?:     UnlistenFn;
  private unlistenExited?:  UnlistenFn;

  @ViewChild('consoleLog') private consoleLogRef?: ElementRef<HTMLDivElement>;

  constructor(
    public auth:  AuthService,
    public tauri: TauriService,
    private ngZone: NgZone,
  ) {}

  // -------------------------------------------------------------------------
  // Lifecycle
  // -------------------------------------------------------------------------

  ngOnInit(): void {
    this.userSub = this.auth.user$.subscribe(user => {
      this.ngZone.run(() => { this.currentUser = user; });
    });
    this.setup();
  }

  ngOnDestroy(): void {
    this.userSub?.unsubscribe();
    this.unlistenState?.();
    this.unlistenLog?.();
    this.unlistenExited?.();
    if (this.statusInterval) clearInterval(this.statusInterval);
  }

  ngAfterViewChecked(): void {
    if (this.shouldScrollLog && this.consoleLogRef) {
      const el = this.consoleLogRef.nativeElement;
      el.scrollTop = el.scrollHeight;
      this.shouldScrollLog = false;
    }
  }

  private async setup(): Promise<void> {
    this.gameState = await invoke<GameState>('get_game_state');

    this.unlistenState = await listen<GameState>('game-state', e => {
      this.ngZone.run(() => { this.gameState = e.payload; });
    });

    this.unlistenLog = await listen<{ line: string; is_error: boolean }>('game-log', e => {
      this.ngZone.run(() => {
        this.logLines.push({ text: e.payload.line, isError: e.payload.is_error });
        if (this.logLines.length > 500) this.logLines.shift();
        this.shouldScrollLog = true;
      });
    });

    this.unlistenExited = await listen<number>('game-exited', e => {
      this.ngZone.run(() => {
        const code = e.payload;
        if (code !== 0) {
          this.logLines.push({ text: `⚠ Game exited with code ${code}`, isError: true });
          this.shouldScrollLog = true;
        }
      });
    });

    // Server status — poll immediately then every 30 s
    await this.refreshServerStatus();
    this.statusInterval = setInterval(() => this.refreshServerStatus(), 30_000);

    // Auto-start
    if (this.tauri.options?.init_on_start && this.currentUser && this.gameState === 'idle') {
      await this.launchGame();
    }
  }

  // -------------------------------------------------------------------------
  // Server status
  // -------------------------------------------------------------------------

  async refreshServerStatus(): Promise<void> {
    this.serverLoading = true;
    try {
      this.serverStatus = await invoke<ServerStatus>('get_server_status');
    } finally {
      this.ngZone.run(() => { this.serverLoading = false; });
    }
  }

  // -------------------------------------------------------------------------
  // Auth modal
  // -------------------------------------------------------------------------

  private askMicrosoftAuth(): Promise<boolean | null> {
    return new Promise(resolve => {
      this.authModalResolve = resolve;
      this.showAuthModal = true;
    });
  }

  onAuthChoice(choice: boolean | null): void {
    this.showAuthModal = false;
    this.authModalResolve?.(choice);
    this.authModalResolve = null;
  }

  // -------------------------------------------------------------------------
  // Game controls
  // -------------------------------------------------------------------------

  async launchGame(): Promise<void> {
    if (!this.currentUser) return;

    const hasToken = await invoke<boolean>('check_ms_auth_state');
    let useMicrosoft: boolean | null = hasToken ? true : null;

    if (!hasToken) {
      useMicrosoft = await this.askMicrosoftAuth();

      // Si el usuario cerró el modal pulsando fuera (null), DETENEMOS LA FUNCIÓN AQUÍ.
      if (useMicrosoft === null) {
        return;
      }
    }

    try {
      this.logLines = [];
      // Llegados a este punto, useMicrosoft es 100% un boolean (true o false).
      // Usamos camelCase porque Tauri lo traduce internamente al snake_case de Rust.
      await invoke('launch_game', { username: this.currentUser.username, useMicrosoft });
    } catch (err: any) {
      await message(String(err), { title: 'Launch Error', kind: 'error' });
    }
  }

  async stopGame(): Promise<void> {
    try {
      await invoke('stop_game');
    } catch (err: any) {
      await message(String(err), { title: 'Stop Error', kind: 'error' });
    }
  }

  clearLog(): void { this.logLines = []; }

  // -------------------------------------------------------------------------
  // Template helpers
  // -------------------------------------------------------------------------

  get isLoggedIn():       boolean { return this.currentUser !== null; }
  get showDebugConsole(): boolean { return !!this.tauri.options?.debug_console; }
  get isIdle():           boolean { return this.gameState === 'idle'; }
  get isRunning():        boolean { return this.gameState === 'running'; }
  get isDownloading():    boolean { return this.gameState === 'downloading'; }
  get canLaunch():        boolean { return this.isLoggedIn && this.isIdle; }
}
