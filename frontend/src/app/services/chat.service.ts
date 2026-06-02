import { Injectable, OnDestroy } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, Subject } from 'rxjs';
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

  constructor(
      private http: HttpClient,
      private authService: AuthService
  ) {}

  connectWebSocket(): void {
    const token = this.authService.getToken();
    const wsUrl = `ws://localhost:3004/ws`;

    this.socket = new WebSocket(wsUrl);

    this.socket.onopen = () => {
      console.log('WebSocket konekcija uspostavljena');
    };

    this.socket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.messageSubject.next(data);
    };

    this.socket.onclose = () => {
      console.log('WebSocket zatvoren');
    };
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
  }

  ngOnDestroy(): void {
    this.disconnectWebSocket();
  }
}