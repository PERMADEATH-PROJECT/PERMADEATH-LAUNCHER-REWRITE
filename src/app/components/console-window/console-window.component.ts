import {
  Component, OnInit, OnDestroy, NgZone, ViewChild, ElementRef, AfterViewChecked
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { LucideAngularModule, Terminal, Square, Skull } from 'lucide-angular';
import { GameState } from '@app/components/play/play.component';

interface LogLine { text: string; isError: boolean; }

@Component({
  selector: 'app-console-window',
  standalone: true,
  imports: [CommonModule, LucideAngularModule],
  templateUrl: './console-window.component.html',
  styleUrls: ['./console-window.component.css'],
})
export class ConsoleWindowComponent implements OnInit, OnDestroy, AfterViewChecked {
  readonly Terminal = Terminal;
  readonly Square   = Square;
  readonly Skull    = Skull;

  gameState: GameState = 'running';
  logLines: LogLine[] = [];
  exitCode: number | null = null;
  private shouldScroll = false;

  private unlistenState?:   UnlistenFn;
  private unlistenLog?:     UnlistenFn;
  private unlistenExited?:  UnlistenFn;

  @ViewChild('consoleEl') private consoleEl?: ElementRef<HTMLDivElement>;

  constructor(private ngZone: NgZone) {}

  async ngOnInit(): Promise<void> {
    this.gameState = await invoke<GameState>('get_game_state');

    this.unlistenState = await listen<GameState>('game-state', e => {
      this.ngZone.run(() => { this.gameState = e.payload; });
    });

    this.unlistenLog = await listen<{ line: string; isError: boolean }>('game-log', e => {
      this.ngZone.run(() => {
        this.logLines.push({ text: e.payload.line, isError: e.payload.isError });
        if (this.logLines.length > 800) this.logLines.shift();
        this.shouldScroll = true;
      });
    });

    this.unlistenExited = await listen<number>('game-exited', e => {
      this.ngZone.run(() => {
        this.exitCode = e.payload;
        this.gameState = 'idle';
      });
    });
  }

  ngOnDestroy(): void {
    this.unlistenState?.();
    this.unlistenLog?.();
    this.unlistenExited?.();
  }

  ngAfterViewChecked(): void {
    if (this.shouldScroll && this.consoleEl) {
      const el = this.consoleEl.nativeElement;
      el.scrollTop = el.scrollHeight;
      this.shouldScroll = false;
    }
  }

  async stopGame(): Promise<void> {
    await invoke('stop_game');
  }

  clearLog(): void { this.logLines = []; }

  get isRunning(): boolean { return this.gameState === 'running'; }
}
