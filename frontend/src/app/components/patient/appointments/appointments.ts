import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatDialogModule } from '@angular/material/dialog';
import { AppointmentService, Appointment } from '../../../services/appointment.service';
import { MediaService } from '../../../services/media.service';

@Component({
  selector: 'app-appointments',
  standalone: true,
  imports: [
    CommonModule, RouterLink, MatCardModule, MatButtonModule,
    MatIconModule, MatChipsModule, MatSnackBarModule, MatDialogModule
  ],
  templateUrl: './appointments.html',
  styleUrl: './appointments.scss'
})
export class Appointments implements OnInit {
  appointments: Appointment[] = [];
  loading = true;

  constructor(
      private appointmentService: AppointmentService,
      private mediaService: MediaService,
      private snackBar: MatSnackBar,
      private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.load();
  }

  load(): void {
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

  cancel(id: string): void {
    this.appointmentService.cancelAppointment(id).subscribe({
      next: () => {
        this.snackBar.open('Termin otkazan', 'Zatvori', { duration: 3000 });
        this.load();
      },
      error: () => this.snackBar.open('Greška pri otkazivanju', 'Zatvori', { duration: 3000 })
    });
  }

  generatePdf(appointmentId: string): void {
  this.mediaService.generatePdf(appointmentId).subscribe({
    next: (mediaFile) => {
      // Automatski downloaduj nakon generisanja
      this.mediaService.downloadFile(mediaFile.id).subscribe({
        next: (blob) => {
          const url = window.URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `potvrda_${appointmentId}.pdf`;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          window.URL.revokeObjectURL(url);
          this.snackBar.open('PDF preuzet!', 'Zatvori', { duration: 3000 });
        },
        error: () => this.snackBar.open('Greška pri preuzimanju PDF-a', 'Zatvori', { duration: 3000 })
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