import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { AppointmentService, Appointment } from '../../../services/appointment.service';

@Component({
  selector: 'app-dentist-appointments',
  standalone: true,
  imports: [CommonModule, MatCardModule, MatButtonModule, MatIconModule, MatChipsModule],
  templateUrl: './appointments.html',
  styleUrl: './appointments.scss'
})
export class Appointments implements OnInit {
  appointments: Appointment[] = [];
  loading = true;

  constructor(
      private appointmentService: AppointmentService,
      private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.appointmentService.getMyAppointments().subscribe({
      next: (data) => {
        this.appointments = data;
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  getStatusLabel(status: string): string {
    const map: any = { scheduled: 'Zakazano', confirmed: 'Potvrđeno', cancelled: 'Otkazano', completed: 'Završeno' };
    return map[status] || status;
  }

  getStatusColor(status: string): string {
    const map: any = { scheduled: 'primary', confirmed: 'accent', cancelled: 'warn', completed: '' };
    return map[status] || '';
  }
}