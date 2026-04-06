import { Routes } from '@angular/router';
import { PlayComponent } from './components/play/play.component';
import { ConfigComponent } from './components/config/config.component';
import { VmComponent } from './components/vm/vm.component';
import { UpdatesComponent } from './components/updates/updates.component';
import { AccountComponent } from './components/account/account.component';
import { ConsoleWindowComponent } from './components/console-window/console-window.component';

export const routes: Routes = [
    { path: '', redirectTo: 'play', pathMatch: 'full' },
    { path: 'play', component: PlayComponent },
    { path: 'config', component: ConfigComponent },
    { path: 'vm', component: VmComponent },
    { path: 'updates', component: UpdatesComponent },
    { path: 'account', component: AccountComponent },
    { path: 'console', component: ConsoleWindowComponent },
];
