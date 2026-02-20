import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { LucideAngularModule, TriangleAlert, Skull, Play, Users, Clock } from 'lucide-angular';

@Component({
    selector: 'app-play',
    standalone: true,
    imports: [CommonModule, LucideAngularModule],
    templateUrl: './play.component.html',
    styleUrls: ['./play.component.css']
})
export class PlayComponent {
    readonly TriangleAlert = TriangleAlert;
    readonly Skull = Skull;
    readonly Play = Play;
    readonly Users = Users;
    readonly Clock = Clock;
}