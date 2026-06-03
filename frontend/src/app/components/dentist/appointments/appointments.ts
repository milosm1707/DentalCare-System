import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { forkJoin } from 'rxjs';
import { AppointmentService, Appointment, Clinic } from '../../../services/appointment.service';
import { AuthService } from '../../../services/auth.service';

interface AppointmentWithDetails extends Appointment {
  patient_name?: string;
  patient_email?: string;
  clinic_name?: string;
  clinic_address?: string;
}

@Component({
  selector: 'app-dentist-appointments',
  standalone: true,
  imports: [CommonModule, MatCardModule, MatButtonModule, MatIconModule, MatChipsModule],
  templateUrl: './appointments.html',
  styleUrl: './appointments.scss'
})
export class Appointments implements OnInit {
  appointments: AppointmentWithDetails[] = [];
  loading = true;

  constructor(
    private appointmentService: AppointmentService,
    private authService: AuthService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    forkJoin({
      appointments: this.appointmentService.getMyAppointments(),
      clinics: this.appointmentService.getClinics()
    }).subscribe({
      next: ({ appointments, clinics }) => {
        this.appointments = appointments.map(apt => {
          const clinic = clinics.find(c => c.id === apt.clinic_id);
          return {
            ...apt,
            clinic_name: clinic?.name || 'Nepoznata ambulanta',
            clinic_address: clinic?.address || ''
          };
        });

        // Dohvati podatke o pacijentima
        const patientIds = [...new Set(this.appointments.map(a => a.patient_id))];
        patientIds.forEach(patientId => {
          this.authService.getUserById(patientId).subscribe({
            next: (user) => {
              this.appointments = this.appointments.map(apt => {
                if (apt.patient_id === patientId) {
                  return {
                    ...apt,
                    patient_name: `${user.first_name} ${user.last_name}`,
                    patient_email: user.email
                  };
                }
                return apt;
              });
              this.cdr.detectChanges();
            }
          });
        });

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