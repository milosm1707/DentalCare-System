import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { AppointmentService, Clinic } from '../../../services/appointment.service';

@Component({
  selector: 'app-schedule',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatFormFieldModule, MatInputModule,
    MatSelectModule, MatSnackBarModule
  ],
  templateUrl: './schedule.html',
  styleUrl: './schedule.scss'
})
export class Schedule implements OnInit {
  clinics: Clinic[] = [];
  showClinicForm = false;
  showSlotForm = false;
  loadingClinic = false;
  loadingSlot = false;

  clinicForm = {
    name: '', address: '', phone: '',
    working_hours_start: '08:00:00',
    working_hours_end: '16:00:00'
  };

  slotForm = {
    clinic_id: '',
    slot_date: '',
    start_time: '09:00:00',
    duration: 'thirty'
  };

  constructor(
      private appointmentService: AppointmentService,
      private snackBar: MatSnackBar,
      private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.loadClinics();
  }

  loadClinics(): void {
  this.appointmentService.getMyClinics().subscribe({
    next: (data) => {
      this.clinics = data;
      this.cdr.detectChanges();
    }
    });
  }

  createClinic(): void {
    if (!this.clinicForm.name || !this.clinicForm.address) return;
    this.loadingClinic = true;

    this.appointmentService.createClinic(this.clinicForm).subscribe({
      next: () => {
        this.loadingClinic = false;
        this.showClinicForm = false;
        this.snackBar.open('Ambulanta kreirana!', 'Zatvori', { duration: 3000 });
        this.clinicForm = {
          name: '', address: '', phone: '',
          working_hours_start: '08:00:00',
          working_hours_end: '16:00:00'
        };
        this.loadClinics();
      },
      error: () => {
        this.loadingClinic = false;
        this.snackBar.open('Greška pri kreiranju ambulante', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  createSlot(): void {
    if (!this.slotForm.clinic_id || !this.slotForm.slot_date) return;
    this.loadingSlot = true;

    this.appointmentService.createSlot(this.slotForm).subscribe({
      next: () => {
        this.loadingSlot = false;
        this.showSlotForm = false;
        this.snackBar.open('Slobodan termin dodan!', 'Zatvori', { duration: 3000 });
        this.slotForm = {
          clinic_id: '', slot_date: '',
          start_time: '09:00:00', duration: 'thirty'
        };
        this.cdr.detectChanges();
      },
      error: () => {
        this.loadingSlot = false;
        this.snackBar.open('Greška pri dodavanju termina', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }
}