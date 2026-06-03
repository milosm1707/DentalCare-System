import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatDividerModule } from '@angular/material/divider';
import { MatTabsModule } from '@angular/material/tabs';
import { AuthService, User } from '../../../services/auth.service';

@Component({
  selector: 'app-profile',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatFormFieldModule, MatInputModule,
    MatSnackBarModule, MatDividerModule, MatTabsModule
  ],
  templateUrl: './profile.html',
  styleUrl: './profile.scss'
})
export class Profile implements OnInit {
  user: User | null = null;
  currentPassword = '';
  newPassword = '';
  confirmPassword = '';
  loadingPassword = false;

  constructor(
    public authService: AuthService,
    private snackBar: MatSnackBar,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.user = this.authService.getCurrentUser();
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
      this.snackBar.open('Nova lozinka mora imati najmanje 6 znakova', 'Zatvori', { duration: 3000 });
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

  getRoleLabel(role: string): string {
    return role === 'dentist' ? 'Stomatolog' : 'Pacijent';
  }
}