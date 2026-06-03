import { Injectable, OnDestroy } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, Subject, BehaviorSubject } from 'rxjs';
import { environment } from '../../environments/environment';
import { AuthService } from './auth.service';

export interface Message {
  id?: string;
  chat_id: string;
  sender_id: string;
  receiver_id: string;
  content: string;
  message_type: string;
  is_read: boolean;
  created_at: string;
}

@Injectable({
  providedIn: 'root'
})
export class ChatService implements OnDestroy {
  private apiUrl = environment.apiUrl;
  private socket: WebSocket | null = null;
  private messageSubject = new Subject<any>();
  public messages$ = this.messageSubject.asObservable();

  // Globalni unread count
  private unreadCountSubject = new BehaviorSubject<number>(0);
  public unreadCount$ = this.unreadCountSubject.asObservable();

  constructor(
    private http: HttpClient,
    private authService: AuthService
  ) {
    // Učitaj sačuvani unread count
    this.loadUnreadCount();

    // Povežise čim se korisnik prijavi
    this.authService.currentUser$.subscribe(user => {
      if (user) {
        this.connectWebSocket();
      } else {
        this.disconnectWebSocket();
        this.unreadCountSubject.next(0);
      }
    });
  }

  private loadUnreadCount(): void {
    const myId = this.authService.getCurrentUser()?.id;
    if (!myId) return;
    const saved = localStorage.getItem(`contacts_${myId}`);
    if (saved) {
      try {
        const contacts = JSON.parse(saved);
        const unread = contacts.filter((c: any) => c.unread).length;
        this.unreadCountSubject.next(unread);
      } catch {
        this.unreadCountSubject.next(0);
      }
    }
  }
  private activeChatContactId: string | null = null;

setActiveChatContact(id: string | null): void {
  this.activeChatContactId = id;
}

getActiveChatContact(): string | null {
  return this.activeChatContactId;
}

  incrementUnread(): void {
    this.unreadCountSubject.next(this.unreadCountSubject.value + 1);
  }

  clearUnread(): void {
    this.unreadCountSubject.next(0);
  }

  decrementUnread(): void {
    const current = this.unreadCountSubject.value;
    if (current > 0) {
      this.unreadCountSubject.next(current - 1);
    }
  }

  connectWebSocket(): void {
  // Ne pravi novu konekciju ako već postoji
  if (this.socket?.readyState === WebSocket.OPEN ||
      this.socket?.readyState === WebSocket.CONNECTING) return;

  const token = this.authService.getToken();
  if (!token) return;

  this.socket = new WebSocket(`ws://localhost:3004/ws?token=${token}`);
  this.socket.onopen = () => console.log('WebSocket connected');
  this.socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    this.messageSubject.next(data);
  };
  this.socket.onclose = () => {
    console.log('WebSocket closed');
    setTimeout(() => {
      if (this.authService.isLoggedIn()) {
        this.connectWebSocket();
      }
    }, 3000);
  };
  this.socket.onerror = (e) => console.error('WebSocket error', e);
}

  sendWsMessage(receiverId: string, content: string): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({
        msg_type: 'text',
        content,
        receiver_id: receiverId
      }));
    }
  }

  getChatHistory(userId: string): Observable<Message[]> {
    return this.http.get<Message[]>(`${this.apiUrl}/messages/${userId}`);
  }

  sendMessage(receiverId: string, content: string): Observable<any> {
    return this.http.post(`${this.apiUrl}/messages`, {
      receiver_id: receiverId,
      content,
      message_type: 'text'
    });
  }

  disconnectWebSocket(): void {
    this.socket?.close();
    this.socket = null;
  }

  ngOnDestroy(): void {
    this.disconnectWebSocket();
  }
}