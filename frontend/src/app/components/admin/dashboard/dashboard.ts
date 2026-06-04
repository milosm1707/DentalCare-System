import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatTableModule } from '@angular/material/table';
import { MatTabsModule } from '@angular/material/tabs';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatChipsModule } from '@angular/material/chips';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { forkJoin } from 'rxjs';
import { AuthService } from '../../../services/auth.service';
import { AppointmentService } from '../../../services/appointment.service';

@Component({
  selector: 'app-admin-dashboard',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatTableModule, MatTabsModule, MatSnackBarModule,
    MatFormFieldModule, MatInputModule, MatSelectModule,
    MatChipsModule, MatSlideToggleModule
  ],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss'
})
export class AdminDashboard implements OnInit {
  // Statistike
  authStats: any = {};
  appointmentStats: any = {};
  loading = true;

  // Korisnici
  users: any[] = [];
  filteredUsers: any[] = [];
  userSearch = '';
  loadingUsers = true;

  // Recenzije
  reviews: any[] = [];
  loadingReviews = true;

  // Edukativni sadržaj
  articles: any[] = [];
  loadingArticles = true;
  showArticleForm = false;
  newArticle = {
    title: '',
    icon: 'article',
    category: 'Savjeti',
    summary: '',
    content: '',
    is_published: true
  };

  categories = ['Osnovna njega', 'Prevencija', 'Bolesti', 'Savjeti', 'Tretmani', 'Estetika'];
  icons = ['brush', 'healing', 'restaurant', 'warning', 'local_hospital', 'event', 'straighten', 'star', 'article'];

  displayedColumns = ['name', 'email', 'role', 'status', 'actions'];

  constructor(
    private authService: AuthService,
    private appointmentService: AppointmentService,
    private snackBar: MatSnackBar,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.loadAll();
  }

  loadAll(): void {
    forkJoin({
      authStats: this.authService.getAdminStats(),
      appointmentStats: this.appointmentService.getAppointmentStats(),
    }).subscribe({
      next: (res) => {
        this.authStats = res.authStats;
        this.appointmentStats = res.appointmentStats;
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });

    this.authService.getAllUsers().subscribe({
      next: (users) => {
        this.users = users;
        this.filteredUsers = users;
        this.loadingUsers = false;
        this.cdr.detectChanges();
      },
      error: () => { this.loadingUsers = false; this.cdr.detectChanges(); }
    });

    this.authService.getAllArticlesAdmin().subscribe({
      next: (articles) => {
        this.articles = articles;
        this.loadingArticles = false;
        this.cdr.detectChanges();
      },
      error: () => { this.loadingArticles = false; this.cdr.detectChanges(); }
    });

    // Dohvati pending recenzije
    this.appointmentService.getPendingReviews().subscribe({
      next: (reviews) => {
        this.reviews = reviews;
        this.loadingReviews = false;
        this.cdr.detectChanges();
      },
      error: () => { this.loadingReviews = false; this.cdr.detectChanges(); }
    });
  }

  onUserSearch(): void {
    const q = this.userSearch.toLowerCase();
    this.filteredUsers = this.users.filter(u =>
      u.first_name.toLowerCase().includes(q) ||
      u.last_name.toLowerCase().includes(q) ||
      u.email.toLowerCase().includes(q)
    );
  }

  blockUser(userId: string): void {
    this.authService.blockUserAdmin(userId).subscribe({
      next: () => {
        const user = this.users.find(u => u.id === userId);
        if (user) user.is_active = false;
        this.snackBar.open('Korisnik blokiran', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  unblockUser(userId: string): void {
    this.authService.unblockUserAdmin(userId).subscribe({
      next: () => {
        const user = this.users.find(u => u.id === userId);
        if (user) user.is_active = true;
        this.snackBar.open('Korisnik deblokiran', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  deleteUser(userId: string): void {
    if (!confirm('Da li ste sigurni da želite obrisati ovog korisnika?')) return;
    this.authService.deleteUserAdmin(userId).subscribe({
      next: () => {
        this.users = this.users.filter(u => u.id !== userId);
        this.filteredUsers = this.filteredUsers.filter(u => u.id !== userId);
        this.snackBar.open('Korisnik obrisan', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  approveReview(reviewId: string): void {
    this.appointmentService.approveReview(reviewId).subscribe({
      next: () => {
        this.reviews = this.reviews.filter(r => r.id !== reviewId);
        this.snackBar.open('Recenzija odobrena!', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  rejectReview(reviewId: string): void {
    this.appointmentService.rejectReview(reviewId).subscribe({
      next: () => {
        this.reviews = this.reviews.filter(r => r.id !== reviewId);
        this.snackBar.open('Recenzija odbijena', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  createArticle(): void {
    if (!this.newArticle.title || !this.newArticle.summary || !this.newArticle.content) {
      this.snackBar.open('Popunite sva obavezna polja', 'Zatvori', { duration: 3000 });
      return;
    }

    this.authService.createArticle(this.newArticle).subscribe({
      next: (article) => {
        this.articles.unshift(article);
        this.showArticleForm = false;
        this.newArticle = { title: '', icon: 'article', category: 'Savjeti', summary: '', content: '', is_published: true };
        this.snackBar.open('Članak kreiran!', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      },
      error: () => this.snackBar.open('Greška pri kreiranju', 'Zatvori', { duration: 3000 })
    });
  }

  toggleArticle(articleId: string): void {
    this.authService.toggleArticle(articleId).subscribe({
      next: () => {
        const article = this.articles.find(a => a.id === articleId);
        if (article) article.is_published = !article.is_published;
        this.cdr.detectChanges();
      }
    });
  }

  deleteArticle(articleId: string): void {
    if (!confirm('Obrisati članak?')) return;
    this.authService.deleteArticle(articleId).subscribe({
      next: () => {
        this.articles = this.articles.filter(a => a.id !== articleId);
        this.snackBar.open('Članak obrisan', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
  }

  getRoleLabel(role: string): string {
    const map: any = { patient: 'Pacijent', dentist: 'Stomatolog', admin: 'Admin' };
    return map[role] || role;
  }

  getStars(rating: number): number[] {
    return Array(rating).fill(0);
  }
}