import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { AppointmentService, Appointment } from '../../../services/appointment.service';
import { AuthService } from '../../../services/auth.service';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, RouterLink, MatCardModule, MatButtonModule, MatIconModule, MatChipsModule],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss'
})
export class Dashboard implements OnInit {
  appointments: Appointment[] = [];
  upcomingCount = 0;
  loading = true;

  constructor(
      private appointmentService: AppointmentService,
      public authService: AuthService,
      private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.appointmentService.getMyAppointments().subscribe({
      next: (data) => {
        this.appointments = data;
        this.upcomingCount = data.filter(a => a.status === 'scheduled').length;
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  getStatusColor(status: string): string {
    const map: any = { scheduled: 'primary', confirmed: 'accent', cancelled: 'warn', completed: '' };
    return map[status] || '';
  }

  getStatusLabel(status: string): string {
    const map: any = { scheduled: 'Zakazano', confirmed: 'Potvrđeno', cancelled: 'Otkazano', completed: 'Završeno' };
    return map[status] || status;
  }
}