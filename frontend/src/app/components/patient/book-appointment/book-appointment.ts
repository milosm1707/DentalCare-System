import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { AppointmentService, Clinic, Slot } from '../../../services/appointment.service';

@Component({
  selector: 'app-book-appointment',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatFormFieldModule, MatInputModule,
    MatSelectModule, MatSnackBarModule
  ],
  templateUrl: './book-appointment.html',
  styleUrl: './book-appointment.scss'
})
export class BookAppointment implements OnInit {
  clinics: Clinic[] = [];
  slots: Slot[] = [];
  selectedClinic: Clinic | null = null;
  selectedSlot: string = '';
  notes: string = '';
  loading = false;
  loadingSlots = false;
  loadingClinics = true;

  constructor(
      private appointmentService: AppointmentService,
      private router: Router,
      private snackBar: MatSnackBar,
      private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.appointmentService.getClinics().subscribe({
      next: (data) => {
        this.clinics = data;
        this.loadingClinics = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loadingClinics = false;
        this.snackBar.open('Greška pri učitavanju ambulanti', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  onClinicSelect(): void {
    if (!this.selectedClinic) return;
    this.loadingSlots = true;
    this.slots = [];
    this.selectedSlot = '';
    this.cdr.detectChanges();

    // Sada šaljemo clinic.id umjesto clinic.dentist_id
    this.appointmentService.getAvailableSlots(this.selectedClinic.id).subscribe({
      next: (data) => {
        this.slots = data;
        this.loadingSlots = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loadingSlots = false;
        this.cdr.detectChanges();
      }
    });
  }

  book(): void {
    if (!this.selectedSlot) return;
    this.loading = true;

    this.appointmentService.bookAppointment(this.selectedSlot, this.notes).subscribe({
      next: () => {
        this.loading = false;
        this.snackBar.open('Termin uspješno zakazan!', 'Zatvori', { duration: 3000 });
        this.router.navigate(['/patient/appointments']);
      },
      error: () => {
        this.loading = false;
        this.snackBar.open('Greška pri zakazivanju', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }
}