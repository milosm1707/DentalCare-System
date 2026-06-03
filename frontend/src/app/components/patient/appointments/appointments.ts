import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatDialogModule } from '@angular/material/dialog';
import { forkJoin } from 'rxjs';
import { AppointmentService, Appointment, Clinic } from '../../../services/appointment.service';
import { MediaService } from '../../../services/media.service';
import { AuthService } from '../../../services/auth.service';
import { FormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';

interface AppointmentWithDetails extends Appointment {
  dentist_name?: string;
  dentist_email?: string;
  clinic_name?: string;
  clinic_address?: string;
}

@Component({
  selector: 'app-appointments',
  standalone: true,
  imports: [
    CommonModule, RouterLink, MatCardModule, MatButtonModule,
    MatIconModule, MatChipsModule, MatSnackBarModule, MatDialogModule,
    FormsModule, MatFormFieldModule, MatInputModule
  ],
  templateUrl: './appointments.html',
  styleUrl: './appointments.scss'
})
export class Appointments implements OnInit {
  appointments: AppointmentWithDetails[] = [];
  clinics: Clinic[] = [];
  loading = true;

  constructor(
    private appointmentService: AppointmentService,
    private mediaService: MediaService,
    private authService: AuthService,
    private snackBar: MatSnackBar,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.load();
  }

  showReviewForm: { [key: string]: boolean } = {};
reviewRating: { [key: string]: number } = {};
reviewComment: { [key: string]: string } = {};

toggleReview(aptId: string): void {
  this.showReviewForm[aptId] = !this.showReviewForm[aptId];
  if (!this.reviewRating[aptId]) this.reviewRating[aptId] = 5;
  if (!this.reviewComment[aptId]) this.reviewComment[aptId] = '';
}

submitReview(apt: AppointmentWithDetails): void {
  const rating = this.reviewRating[apt.id];
  const comment = this.reviewComment[apt.id];

  this.appointmentService.createReview(
    apt.dentist_id,
    apt.id,
    rating,
    comment
  ).subscribe({
    next: () => {
      this.snackBar.open('Recenzija poslana na odobrenje!', 'Zatvori', { duration: 3000 });
      this.showReviewForm[apt.id] = false;
    },
    error: () => this.snackBar.open('Greška ili ste već ostavili recenziju', 'Zatvori', { duration: 3000 })
  });
  } 

  getStars(rating: number): number[] {
    return Array(rating).fill(0);
  }

  load(): void {
    forkJoin({
      appointments: this.appointmentService.getMyAppointments(),
      clinics: this.appointmentService.getClinics()
    }).subscribe({
      next: ({ appointments, clinics }) => {
        this.clinics = clinics;
        this.appointments = appointments.map(apt => {
          const clinic = clinics.find(c => c.id === apt.clinic_id);
          return {
            ...apt,
            clinic_name: clinic?.name || 'Nepoznata ambulanta',
            clinic_address: clinic?.address || ''
          };
        });

        // Dohvati podatke o doktorima
        const dentistIds = [...new Set(this.appointments.map(a => a.dentist_id))];
        dentistIds.forEach(dentistId => {
          this.authService.getUserById(dentistId).subscribe({
            next: (user) => {
              this.appointments = this.appointments.map(apt => {
                if (apt.dentist_id === dentistId) {
                  return {
                    ...apt,
                    dentist_name: `${user.first_name} ${user.last_name}`,
                    dentist_email: user.email
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

  cancel(id: string): void {
    this.appointmentService.cancelAppointment(id).subscribe({
      next: () => {
        this.snackBar.open('Termin otkazan', 'Zatvori', { duration: 3000 });
        this.load();
      },
      error: () => this.snackBar.open('Greška pri otkazivanju', 'Zatvori', { duration: 3000 })
    });
  }

  generatePdf(apt: AppointmentWithDetails): void {
  const currentUser = this.authService.getCurrentUser();
  if (!currentUser) return;

  // Ako podaci o doktoru nisu učitani, dohvati ih prvo
  if (!apt.dentist_email) {
    this.authService.getUserById(apt.dentist_id).subscribe({
      next: (user) => {
        apt.dentist_name = `${user.first_name} ${user.last_name}`;
        apt.dentist_email = user.email;
        this.buildAndDownloadPdf(apt, currentUser);
      }
    });
  } else {
    this.buildAndDownloadPdf(apt, currentUser);
  }
}

private buildAndDownloadPdf(apt: AppointmentWithDetails, currentUser: any): void {
  const pdfData = {
    patient_name: `${currentUser.first_name} ${currentUser.last_name}`,
    patient_email: currentUser.email,
    dentist_name: apt.dentist_name || 'Nepoznat',
    dentist_email: apt.dentist_email || 'Nepoznat',
    clinic_name: apt.clinic_name || 'Nepoznata ambulanta',
    clinic_address: apt.clinic_address || '',
    appointment_date: apt.appointment_date,
    start_time: apt.start_time,
    duration: apt.duration,
    notes: apt.notes || null
  };

  this.mediaService.generatePdf(apt.id, pdfData).subscribe({
    next: (mediaFile) => {
      this.mediaService.downloadFile(mediaFile.id).subscribe({
        next: (blob) => {
          const url = window.URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `potvrda_${apt.appointment_date}.pdf`;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          window.URL.revokeObjectURL(url);
          this.snackBar.open('PDF preuzet!', 'Zatvori', { duration: 3000 });
        },
        error: () => this.snackBar.open('Greška pri preuzimanju', 'Zatvori', { duration: 3000 })
      });
    },
    error: () => this.snackBar.open('Greška pri generisanju PDF-a', 'Zatvori', { duration: 3000 })
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
