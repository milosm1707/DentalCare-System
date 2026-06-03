import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { AppointmentService, Review } from '../../../services/appointment.service';

@Component({
  selector: 'app-dentist-reviews',
  standalone: true,
  imports: [CommonModule, MatCardModule, MatIconModule, MatChipsModule],
  templateUrl: './reviews.html',
  styleUrl: './reviews.scss'
})
export class DentistReviews implements OnInit {
  reviews: Review[] = [];
  loading = true;
  averageRating = 0;

  constructor(
    private appointmentService: AppointmentService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.appointmentService.getMyReviews().subscribe({
      next: (data) => {
        this.reviews = data;
        if (data.length > 0) {
          const approved = data.filter(r => r.status === 'approved');
          this.averageRating = approved.length > 0
            ? approved.reduce((sum, r) => sum + r.rating, 0) / approved.length
            : 0;
        }
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  getStars(rating: number): number[] {
    return Array(rating).fill(0);
  }

  getEmptyStars(rating: number): number[] {
    return Array(5 - rating).fill(0);
  }

  getStatusLabel(status: string): string {
    const map: any = { pending: 'Na čekanju', approved: 'Odobreno', rejected: 'Odbijeno' };
    return map[status] || status;
  }

  getStatusColor(status: string): string {
    const map: any = { pending: 'accent', approved: 'primary', rejected: 'warn' };
    return map[status] || '';
  }
}