import { Component, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { AuthService } from '../../../services/auth.service';

@Component({
  selector: 'app-register',
  standalone: true,
  imports: [
    CommonModule, FormsModule, RouterLink, MatCardModule,
    MatFormFieldModule, MatInputModule, MatButtonModule,
    MatSelectModule, MatSnackBarModule
  ],
  templateUrl: './register.html',
  styleUrl: './register.scss'
})
export class Register {
  email = '';
  password = '';
  first_name = '';
  last_name = '';
  phone = '';
  role = 'patient';
  loading = false;

  constructor(
      private authService: AuthService,
      private router: Router,
      private snackBar: MatSnackBar,
      private cdr: ChangeDetectorRef
  ) {}

  onSubmit(): void {
    if (!this.email || !this.password || !this.first_name || !this.last_name) return;
    this.loading = true;

    this.authService.register({
      email: this.email,
      password: this.password,
      first_name: this.first_name,
      last_name: this.last_name,
      phone: this.phone,
      role: this.role
    }).subscribe({
      next: (res) => {
        this.loading = false;
        this.cdr.detectChanges();
        this.snackBar.open('Registracija uspješna!', 'Zatvori', { duration: 3000 });
        if (res.user.role === 'dentist') {
          this.router.navigate(['/dentist/dashboard']);
        } else {
          this.router.navigate(['/patient/dashboard']);
        }
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
        this.snackBar.open('Greška. Email možda već postoji.', 'Zatvori', { duration: 3000 });
      }
    });
  }
}