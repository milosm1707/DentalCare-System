import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import { Router, RouterOutlet } from '@angular/router';
import { CommonModule } from '@angular/common';
import { Navbar } from './components/shared/navbar/navbar';
import { ChatService } from './services/chat.service';
import { AuthService } from './services/auth.service';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, Navbar, CommonModule, MatSnackBarModule],
  templateUrl: './app.html',
  styleUrl: './app.scss'
})
export class App implements OnInit {
  title = 'frontend';

  constructor(
    private chatService: ChatService,
    private authService: AuthService,
    private snackBar: MatSnackBar,
    private router: Router,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    // Povežise odmah ako je korisnik već prijavljen
    if (this.authService.isLoggedIn()) {
      this.chatService.connectWebSocket();
    }

    // Slušaj poruke globalno — uvijek aktivno
    this.chatService.messages$.subscribe(msg => {
  if (!msg.content || msg.status) return;

  const myId = this.authService.getCurrentUser()?.id;
  if (msg.sender_id && msg.sender_id !== myId) {
    
    // Provjeri da li je korisnik već u chatu sa ovom osobom
    const currentUrl = this.router.url;
    const activeChatId = this.chatService.getActiveChatContact();
    
    if (currentUrl === '/chat' && activeChatId === msg.sender_id) {
      // Korisnik je u aktivnom chatu sa pošiljaocem — ne prikazuj notifikaciju
      return;
    }

    this.chatService.incrementUnread();

    const snackRef = this.snackBar.open(
      '💬 Nova poruka',
      'Otvori chat',
      {
        duration: 5000,
        horizontalPosition: 'right',
        verticalPosition: 'top',
        panelClass: ['chat-notification']
      }
    );

    snackRef.onAction().subscribe(() => {
      this.router.navigate(['/chat']);
    });
  }
});
  }
}