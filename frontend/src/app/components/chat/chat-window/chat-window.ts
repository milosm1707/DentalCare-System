import { Component, OnInit, OnDestroy, ViewChild, ElementRef, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatListModule } from '@angular/material/list';
import { MatDividerModule } from '@angular/material/divider';
import { MatBadgeModule } from '@angular/material/badge';
import { ChatService, Message } from '../../../services/chat.service';
import { AuthService, User } from '../../../services/auth.service';

interface ChatContact {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  unread: boolean;
}

@Component({
  selector: 'app-chat-window',
  standalone: true,
  imports: [
    CommonModule, FormsModule, MatCardModule, MatButtonModule,
    MatIconModule, MatFormFieldModule, MatInputModule,
    MatListModule, MatDividerModule, MatBadgeModule
  ],
  templateUrl: './chat-window.html',
  styleUrl: './chat-window.scss'
})
export class ChatWindow implements OnInit, OnDestroy {
  @ViewChild('messagesEnd') messagesEnd!: ElementRef;

  searchQuery = '';
  searchResults: User[] = [];
  searchTimeout: any;
  contacts: ChatContact[] = [];
  activeContact: ChatContact | null = null;
  messages: Message[] = [];
  newMessage = '';
  loadingMessages = false;
  private myId = '';

  constructor(
    public authService: AuthService,
    private chatService: ChatService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
  this.myId = this.authService.getCurrentUser()?.id || '';

  // Učitaj kontakte
  const saved = localStorage.getItem(`contacts_${this.myId}`);
  if (saved) {
    try {
      this.contacts = JSON.parse(saved);
      // Izračunaj unread iz kontakata
      const unread = this.contacts.filter(c => c.unread).length;
      if (unread === 0) this.chatService.clearUnread();
    } catch {
      this.contacts = [];
    }
  }

  // Slušaj dolazne poruke
  this.chatService.messages$.subscribe(msg => {
    if (!msg.content || msg.status) return;

    if (msg.sender_id && msg.sender_id !== this.myId) {
      this.authService.getUserById(msg.sender_id).subscribe({
        next: (user) => {
          const contact: ChatContact = {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            unread: this.activeContact?.id !== msg.sender_id
          };

          this.addToContacts(contact);

          if (this.activeContact?.id === msg.sender_id) {
            this.messages.push({
              chat_id: '',
              sender_id: msg.sender_id,
              receiver_id: this.myId,
              content: msg.content,
              message_type: 'text',
              is_read: false,
              created_at: msg.created_at || new Date().toISOString()
            });
            this.cdr.detectChanges();
            this.scrollToBottom();
          }
        },
        error: () => {
          if (this.activeContact?.id === msg.sender_id) {
            this.messages.push({
              chat_id: '',
              sender_id: msg.sender_id,
              receiver_id: this.myId,
              content: msg.content,
              message_type: 'text',
              is_read: false,
              created_at: msg.created_at || new Date().toISOString()
            });
            this.cdr.detectChanges();
            this.scrollToBottom();
          }
        }
      });
    }
  });
}

  private addToContacts(newContact: ChatContact): void {
  const existing = this.contacts.find(c => c.id === newContact.id);
  if (existing) {
    // Ažuriraj unread samo ako je nova poruka
    if (newContact.unread) {
      existing.unread = true;
    }
  } else {
    this.contacts.unshift(newContact);
  }
  this.saveContacts();
  this.cdr.detectChanges();
}

  private saveContacts(): void {
    localStorage.setItem(`contacts_${this.myId}`, JSON.stringify(this.contacts));
  }

  onSearch(): void {
    clearTimeout(this.searchTimeout);
    if (!this.searchQuery.trim()) {
      this.searchResults = [];
      this.cdr.detectChanges();
      return;
    }
    this.searchTimeout = setTimeout(() => {
      this.authService.searchUsers(this.searchQuery).subscribe({
        next: (users) => {
          this.searchResults = users.filter(u => u.id !== this.myId);
          this.cdr.detectChanges();
        }
      });
    }, 300);
  }

  selectContact(user: User): void {
    const contact: ChatContact = {
      id: user.id,
      email: user.email,
      first_name: user.first_name,
      last_name: user.last_name,
      unread: false
    };

    this.addToContacts(contact);
    this.openChat(contact);
    this.searchQuery = '';
    this.searchResults = [];
    this.cdr.detectChanges();
  }

  openChat(contact: ChatContact): void {
  this.activeContact = contact;
  this.chatService.setActiveChatContact(contact.id);

  // Osvježi podatke o kontaktu
  this.authService.getUserById(contact.id).subscribe({
    next: (user) => {
      const updatedContact: ChatContact = {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        unread: false
      };

      // Ažuriraj u listi kontakata
      const idx = this.contacts.findIndex(c => c.id === contact.id);
      if (idx !== -1) {
        this.contacts[idx] = { ...updatedContact, unread: false };
      }

      this.activeContact = updatedContact;
      this.saveContacts();
      this.cdr.detectChanges();
    }
  });

  // Označi kao pročitano
  const existing = this.contacts.find(c => c.id === contact.id);
  if (existing && existing.unread) {
    existing.unread = false;
    this.saveContacts();
    this.chatService.decrementUnread();
  }

  const stillUnread = this.contacts.filter(c => c.unread).length;
  if (stillUnread === 0) {
    this.chatService.clearUnread();
  }

  this.messages = [];
  this.loadingMessages = true;
  this.cdr.detectChanges();

  this.chatService.getChatHistory(contact.id).subscribe({
    next: (msgs) => {
      this.messages = msgs;
      this.loadingMessages = false;
      this.cdr.detectChanges();
      this.scrollToBottom();
    },
    error: () => {
      this.loadingMessages = false;
      this.cdr.detectChanges();
    }
  });
}

  send(): void {
    if (!this.newMessage.trim() || !this.activeContact) return;

    const content = this.newMessage;
    const receiverId = this.activeContact.id;
    this.newMessage = '';

    this.messages.push({
      chat_id: '',
      sender_id: this.myId,
      receiver_id: receiverId,
      content,
      message_type: 'text',
      is_read: false,
      created_at: new Date().toISOString()
    });
    this.cdr.detectChanges();
    this.scrollToBottom();

    this.chatService.sendMessage(receiverId, content).subscribe({
      error: () => {
        this.messages = this.messages.filter(m =>
          !(m.sender_id === this.myId && m.content === content)
        );
        this.newMessage = content;
        this.cdr.detectChanges();
      }
    });
  }

  isMyMessage(msg: Message): boolean {
    return msg.sender_id === this.myId;
  }

  private scrollToBottom(): void {
    setTimeout(() => {
      this.messagesEnd?.nativeElement?.scrollIntoView({ behavior: 'smooth' });
    }, 100);
  }

  ngOnDestroy(): void {
  this.chatService.setActiveChatContact(null);
}
}