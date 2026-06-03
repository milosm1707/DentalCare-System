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
import { MatTabsModule } from '@angular/material/tabs';
import { MatDividerModule } from '@angular/material/divider';
import { AuthService, DentistProfileData } from '../../../services/auth.service';

@Component({
  selector: 'app-dentist-profile',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatFormFieldModule, MatInputModule,
    MatSelectModule, MatSnackBarModule, MatTabsModule, MatDividerModule
  ],
  templateUrl: './profile.html',
  styleUrl: './profile.scss'
})
export class DentistProfile implements OnInit {
  profile: any = {};
  loading = true;
  saving = false;

  currentPassword = '';
  newPassword = '';
  confirmPassword = '';
  loadingPassword = false;

  specializations = [
    'Oralna hirurgija',
    'Ortodoncija',
    'Parodontologija',
    'Dječija stomatologija',
    'Protetika',
    'Endodoncija',
    'Estetska stomatologija',
    'Implantologija',
    'Opšta stomatologija'
  ];

  constructor(
    private authService: AuthService,
    private snackBar: MatSnackBar,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    const user = this.authService.getCurrentUser();
    if (!user) return;

    this.authService.getDentistProfile(user.id).subscribe({
      next: (data) => {
        this.profile = { ...data };
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  saveProfile(): void {
  this.saving = true;
  this.authService.updateDentistProfile(this.profile).subscribe({
    next: (res) => {
      this.saving = false;
      // Ažuriraj lokalne podatke
      if (res.user) {
        this.authService.updateLocalUser(res.user);
      }
      this.snackBar.open('Profil uspješno ažuriran!', 'Zatvori', { duration: 3000 });
      this.cdr.detectChanges();
    },
    error: () => {
      this.saving = false;
      this.snackBar.open('Greška pri ažuriranju profila', 'Zatvori', { duration: 3000 });
      this.cdr.detectChanges();
    }
  });
}

  changePassword(): void {
    if (!this.currentPassword || !this.newPassword || !this.confirmPassword) {
      this.snackBar.open('Popunite sva polja', 'Zatvori', { duration: 3000 });
      return;
    }
    if (this.newPassword !== this.confirmPassword) {
      this.snackBar.open('Nove lozinke se ne podudaraju', 'Zatvori', { duration: 3000 });
      return;
    }
    if (this.newPassword.length < 6) {
      this.snackBar.open('Lozinka mora imati najmanje 6 znakova', 'Zatvori', { duration: 3000 });
      return;
    }

    this.loadingPassword = true;
    this.authService.changePassword(this.currentPassword, this.newPassword).subscribe({
      next: () => {
        this.loadingPassword = false;
        this.currentPassword = '';
        this.newPassword = '';
        this.confirmPassword = '';
        this.snackBar.open('Lozinka uspješno promijenjena!', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      },
      error: () => {
        this.loadingPassword = false;
        this.snackBar.open('Pogrešna trenutna lozinka', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }
}