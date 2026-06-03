import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatChipsModule } from '@angular/material/chips';
import { MediaService, MediaFile } from '../../../services/media.service';

@Component({
  selector: 'app-xray',
  standalone: true,
  imports: [
    CommonModule, MatCardModule, MatButtonModule, MatIconModule,
    MatSnackBarModule, MatProgressBarModule, MatChipsModule
  ],
  templateUrl: './xray.html',
  styleUrl: './xray.scss'
})
export class Xray implements OnInit {
  files: MediaFile[] = [];
  loading = true;
  uploading = false;
  dragOver = false;

  constructor(
    private mediaService: MediaService,
    private snackBar: MatSnackBar,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.loadFiles();
  }

  loadFiles(): void {
    this.mediaService.getMyFiles().subscribe({
      next: (data) => {
        this.files = data.filter(f =>
          f.media_type === 'xray' || f.media_type === 'profile_image' || f.media_type === 'chat_image'
        );
        this.loading = false;
        this.cdr.detectChanges();
      },
      error: () => {
        this.loading = false;
        this.cdr.detectChanges();
      }
    });
  }

  onFileSelected(event: any): void {
    const file = event.target.files[0];
    if (file) this.uploadFile(file);
  }

  onDrop(event: DragEvent): void {
    event.preventDefault();
    this.dragOver = false;
    const file = event.dataTransfer?.files[0];
    if (file) this.uploadFile(file);
  }

  onDragOver(event: DragEvent): void {
    event.preventDefault();
    this.dragOver = true;
  }

  onDragLeave(): void {
    this.dragOver = false;
  }

  uploadFile(file: File): void {
    const allowed = ['image/jpeg', 'image/png', 'image/webp', 'application/pdf'];
    if (!allowed.includes(file.type)) {
      this.snackBar.open('Dozvoljeni formati: JPG, PNG, WEBP, PDF', 'Zatvori', { duration: 3000 });
      return;
    }

    const maxSize = 10 * 1024 * 1024; // 10MB
    if (file.size > maxSize) {
      this.snackBar.open('Maksimalna veličina fajla je 10MB', 'Zatvori', { duration: 3000 });
      return;
    }

    this.uploading = true;
    this.cdr.detectChanges();

    this.mediaService.uploadFile(file, 'xray').subscribe({
      next: () => {
        this.uploading = false;
        this.snackBar.open('RTG snimak uspješno uploadovan!', 'Zatvori', { duration: 3000 });
        this.loadFiles();
      },
      error: () => {
        this.uploading = false;
        this.snackBar.open('Greška pri uploadu', 'Zatvori', { duration: 3000 });
        this.cdr.detectChanges();
      }
    });
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

  getFileIcon(mediaType: string): string {
    switch(mediaType) {
      case 'xray': return 'radiology';
      case 'pdf_confirmation': return 'picture_as_pdf';
      default: return 'image';
    }
  }

  getFileTypeLabel(mediaType: string): string {
    switch(mediaType) {
      case 'xray': return 'RTG snimak';
      case 'profile_image': return 'Slika';
      case 'chat_image': return 'Chat slika';
      case 'pdf_confirmation': return 'PDF potvrda';
      default: return mediaType;
    }
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