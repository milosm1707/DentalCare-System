import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BehaviorSubject, Observable, tap } from 'rxjs';
import { Router } from '@angular/router';
import { environment } from '../../environments/environment';

export interface User {
  id: string;
  email: string;
  role: string;
  first_name: string;
  last_name: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

export interface DentistProfileData {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  phone?: string;
  specialization?: string;
  bio?: string;
  clinic_name?: string;
  clinic_address?: string;
  working_hours_start?: string;
  working_hours_end?: string;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private apiUrl = `${environment.apiUrl}/auth`;
  private currentUserSubject = new BehaviorSubject<User | null>(null);
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor(private http: HttpClient, private router: Router) {
    const savedUser = localStorage.getItem('user');
    if (savedUser) {
      this.currentUserSubject.next(JSON.parse(savedUser));
    }
  }
  getUserById(id: string): Observable<User> {
  return this.http.get<User>(`${this.apiUrl}/users/${id}`);
  }

  register(data: any): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.apiUrl}/register`, data).pipe(
        tap(res => this.saveSession(res))
    );
  }

  login(email: string, password: string): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.apiUrl}/login`, { email, password }).pipe(
        tap(res => this.saveSession(res))
    );
  }

  logout(): void {
    localStorage.removeItem('token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('user');
    this.currentUserSubject.next(null);
    this.router.navigate(['/login']);
  }

  getToken(): string | null {
    return localStorage.getItem('token');
  }

  isLoggedIn(): boolean {
    return !!this.getToken();
  }

  getCurrentUser(): User | null {
    return this.currentUserSubject.value;
  }

  getRole(): string | null {
    return this.getCurrentUser()?.role || null;
  }

  private saveSession(res: AuthResponse): void {
    localStorage.setItem('token', res.access_token);
    localStorage.setItem('refresh_token', res.refresh_token);
    localStorage.setItem('user', JSON.stringify(res.user));
    this.currentUserSubject.next(res.user);
  }
  searchUsers(query: string): Observable<User[]> {
    return this.http.get<User[]>(`${this.apiUrl}/users/search?q=${query}`);
  }
  changePassword(currentPassword: string, newPassword: string): Observable<any> {
  return this.http.post(`${this.apiUrl}/change-password`, {
    current_password: currentPassword,
    new_password: newPassword
  });
}
getDentistProfile(userId: string): Observable<DentistProfileData> {
  return this.http.get<DentistProfileData>(`${this.apiUrl}/users/${userId}/dentist-profile`);
}

updateDentistProfile(data: any): Observable<any> {
  return this.http.put(`${this.apiUrl}/dentist-profile`, data);
}
updateLocalUser(user: User): void {
  localStorage.setItem('user', JSON.stringify(user));
  this.currentUserSubject.next(user);
}
// Admin metode
getAdminStats(): Observable<any> {
  return this.http.get(`${this.apiUrl}/admin/stats`);
}

getAllUsers(): Observable<any[]> {
  return this.http.get<any[]>(`${this.apiUrl}/admin/users`);
}

blockUserAdmin(userId: string): Observable<any> {
  return this.http.post(`${this.apiUrl}/admin/users/${userId}/block`, {});
}

unblockUserAdmin(userId: string): Observable<any> {
  return this.http.post(`${this.apiUrl}/admin/users/${userId}/unblock`, {});
}

deleteUserAdmin(userId: string): Observable<any> {
  return this.http.delete(`${this.apiUrl}/admin/users/${userId}`);
}

getAllArticlesAdmin(): Observable<any[]> {
  return this.http.get<any[]>(`${this.apiUrl}/admin/articles`);
}

createArticle(data: any): Observable<any> {
  return this.http.post(`${this.apiUrl}/admin/articles`, data);
}

deleteArticle(id: string): Observable<any> {
  return this.http.delete(`${this.apiUrl}/admin/articles/${id}`);
}

toggleArticle(id: string): Observable<any> {
  return this.http.post(`${this.apiUrl}/admin/articles/${id}/toggle`, {});
}

getArticles(): Observable<any[]> {
  return this.http.get<any[]>(`${environment.apiUrl}/articles`);
}
}