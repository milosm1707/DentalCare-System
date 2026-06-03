import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink, Router } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { FormsModule } from '@angular/forms';
import { AppointmentService, Clinic } from '../../../services/appointment.service';
import { AuthService } from '../../../services/auth.service';

@Component({
  selector: 'app-dentists',
  standalone: true,
  imports: [
    CommonModule, RouterLink, MatCardModule, MatButtonModule,
    MatIconModule, MatChipsModule, MatFormFieldModule,
    MatInputModule, FormsModule
  ],
  templateUrl: './dentists.html',
  styleUrl: './dentists.scss'
})
export class Dentists implements OnInit {
  clinics: Clinic[] = [];
  filtered: Clinic[] = [];
  searchQuery = '';
  loading = true;

  constructor(
    private appointmentService: AppointmentService,
    private authService: AuthService,
    private router: Router,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.appointmentService.getClinics().subscribe({
      next: (data) => {
        this.clinics = data;
        this.filtered = data;
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  onSearch(): void {
    const q = this.searchQuery.toLowerCase();
    this.filtered = this.clinics.filter(c =>
      c.name.toLowerCase().includes(q) ||
      c.address.toLowerCase().includes(q)
    );
  }

  bookAppointment(): void {
    if (this.authService.isLoggedIn()) {
      this.router.navigate(['/patient/book']);
    } else {
      this.router.navigate(['/register']);
    }
  }
}