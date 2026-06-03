import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatChipsModule } from '@angular/material/chips';
import { forkJoin } from 'rxjs';
import { MediaService, MediaFile } from '../../../services/media.service';
import { AppointmentService } from '../../../services/appointment.service';
import { AuthService, User } from '../../../services/auth.service';

interface PatientWithFiles {
  patient_id: string;
  patient_name: string;
  patient_email: string;
  files: MediaFile[];
  expanded: boolean;
}

@Component({
  selector: 'app-dentist-xray',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatSnackBarModule, MatFormFieldModule,
    MatInputModule, MatChipsModule
  ],
  templateUrl: './xray.html',
  styleUrl: './xray.scss'
})
export class DentistXray implements OnInit {
  patients: PatientWithFiles[] = [];
  loading = true;

  // Pretraga
  searchQuery = '';
  searchResults: User[] = [];
  searchTimeout: any;
  searching = false;

  constructor(
    private mediaService: MediaService,
    private appointmentService: AppointmentService,
    private authService: AuthService,
    private snackBar: MatSnackBar,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.appointmentService.getMyAppointments().subscribe({
      next: (appointments) => {
        const patientIds = [...new Set(appointments.map(a => a.patient_id))];

        if (patientIds.length === 0) {
          this.loading = false;
          this.cdr.detectChanges();
          return;
        }

        const requests = patientIds.map(patientId =>
          forkJoin({
            user: this.authService.getUserById(patientId),
            files: this.mediaService.getPatientFiles(patientId)
          })
        );

        forkJoin(requests).subscribe({
          next: (results) => {
            this.patients = results
              .filter(r => r.files.length > 0)
              .map(r => ({
                patient_id: r.user.id,
                patient_name: `${r.user.first_name} ${r.user.last_name}`,
                patient_email: r.user.email,
                files: r.files,
                expanded: false
              }));
            this.loading = false;
            this.cdr.detectChanges();
          },
          error: () => {
            this.loading = false;
            this.cdr.detectChanges();
          }
        });
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  onSearch(): void {
    clearTimeout(this.searchTimeout);
    if (!this.searchQuery.trim()) {
      this.searchResults = [];
      this.cdr.detectChanges();
      return;
    }
    this.searchTimeout = setTimeout(() => {
      this.authService.searchUsers(this.searchQuery).subscribe({
        next: (users) => {
          this.searchResults = users.filter(u => u.role === 'patient');
          this.cdr.detectChanges();
        }
      });
    }, 300);
  }

  loadPatientFiles(user: User): void {
    this.searching = true;
    this.searchResults = [];
    this.searchQuery = '';

    // Provjeri da li već postoji u listi
    if (this.patients.find(p => p.patient_id === user.id)) {
      this.searching = false;
      this.snackBar.open('Pacijent je već u listi', 'Zatvori', { duration: 2000 });
      this.cdr.detectChanges();
      return;
    }

    this.mediaService.getPatientFiles(user.id).subscribe({
      next: (files) => {
        this.searching = false;
        if (files.length === 0) {
          this.snackBar.open('Pacijent nema uploadovanih RTG snimaka', 'Zatvori', { duration: 3000 });
        } else {
          this.patients.unshift({
            patient_id: user.id,
            patient_name: `${user.first_name} ${user.last_name}`,
            patient_email: user.email,
            files,
            expanded: true
          });
        }
        this.cdr.detectChanges();
      },
      error: () => {
        this.searching = false;
        this.snackBar.open('Greška pri učitavanju snimaka', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  toggleExpand(patient: PatientWithFiles): void {
    patient.expanded = !patient.expanded;
  }

  downloadFile(file: MediaFile): void {
    this.mediaService.downloadFile(file.id).subscribe({
      next: (blob) => {
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = file.file_name;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        window.URL.revokeObjectURL(url);
      },
      error: () => this.snackBar.open('Greška pri preuzimanju', 'Zatvori', { duration: 3000 })
    });
  }

  formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }

  formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('sr-RS', {
      day: '2-digit', month: '2-digit', year: 'numeric',
      hour: '2-digit', minute: '2-digit'
    });
  }
}