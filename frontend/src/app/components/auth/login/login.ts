import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { AuthService } from '../../../services/auth.service';
import { ChangeDetectorRef } from '@angular/core';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterLink,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatSnackBarModule
  ],
  templateUrl: './login.html',
  styleUrl: './login.scss'
})
export class Login {
  email = '';
  password = '';
  loading = false;

  constructor(
      private authService: AuthService,
      private router: Router,
      private snackBar: MatSnackBar,
      private cdr: ChangeDetectorRef
  ) {}

  onSubmit(): void {
    if (!this.email || !this.password) return;
    this.loading = true;

    this.authService.login(this.email, this.password).subscribe({
      next: (res) => {
        this.loading = false;
        this.cdr.detectChanges();
        if (res.user.role === 'dentist') {
          this.router.navigate(['/dentist/dashboard']);
        } else if (res.user.role === 'admin') {
          this.router.navigate(['/admin']);
        } else {
          this.router.navigate(['/patient/dashboard']);
        }
      },
      error: (err) => {
        this.loading = false;
        this.snackBar.open('Pogrešan email ili lozinka', 'Zatvori', { duration: 3000 });
      }
    });
  }
}