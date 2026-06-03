import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';

export interface Clinic {
  id: string;
  dentist_id: string;
  name: string;
  address: string;
  phone?: string;
  working_hours_start: string;
  working_hours_end: string;
}

export interface Slot {
  id: string;
  dentist_id: string;
  clinic_id: string;
  slot_date: string;
  start_time: string;
  duration: string;
  is_available: boolean;
}

export interface Appointment {
  id: string;
  patient_id: string;
  dentist_id: string;
  clinic_id: string;
  slot_id: string;
  status: string;
  notes?: string;
  appointment_date: string;
  start_time: string;
  duration: string;
}

@Injectable({
  providedIn: 'root'
})
export class AppointmentService {
  private apiUrl = environment.apiUrl;

  constructor(private http: HttpClient) {}

  getClinics(): Observable<Clinic[]> {
    return this.http.get<Clinic[]>(`${this.apiUrl}/clinics`);
  }

  createClinic(data: any): Observable<Clinic> {
    return this.http.post<Clinic>(`${this.apiUrl}/clinics`, data);
  }

  getAvailableSlots(clinicId: string): Observable<Slot[]> {
    return this.http.get<Slot[]>(`${this.apiUrl}/slots/${clinicId}`);
  }

  createSlot(data: any): Observable<Slot> {
    return this.http.post<Slot>(`${this.apiUrl}/slots`, data);
  }

  bookAppointment(slotId: string, notes?: string): Observable<Appointment> {
    return this.http.post<Appointment>(`${this.apiUrl}/appointments`, {
      slot_id: slotId,
      notes
    });
  }

  getMyAppointments(): Observable<Appointment[]> {
    return this.http.get<Appointment[]>(`${this.apiUrl}/appointments/my`);
  }

  cancelAppointment(id: string): Observable<Appointment> {
    return this.http.delete<Appointment>(`${this.apiUrl}/appointments/${id}/cancel`);
  }
}