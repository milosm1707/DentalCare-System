import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';

export interface MediaFile {
  id: string;
  owner_id: string;
  file_name: string;
  file_path: string;
  media_type: string;
  mime_type: string;
  file_size: number;
  appointment_id?: string;
  created_at: string;
}

@Injectable({
  providedIn: 'root'
})
export class MediaService {
  private apiUrl = environment.apiUrl;

  constructor(private http: HttpClient) {}

  uploadFile(file: File): Observable<MediaFile> {
    const formData = new FormData();
    formData.append('file', file);
    return this.http.post<MediaFile>(`${this.apiUrl}/upload`, formData);
  }

  getMyFiles(): Observable<MediaFile[]> {
    return this.http.get<MediaFile[]>(`${this.apiUrl}/files`);
  }

  downloadFile(fileId: string): Observable<Blob> {
    return this.http.get(`${this.apiUrl}/files/${fileId}/download`, {
      responseType: 'blob'
    });
  }

  generatePdf(appointmentId: string): Observable<MediaFile> {
    return this.http.post<MediaFile>(`${this.apiUrl}/pdf/${appointmentId}`, {});
  }
}