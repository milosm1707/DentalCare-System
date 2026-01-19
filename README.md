DentaCare - Mikroservisna Aplikacija za Stomatološke Ambulante
Opis Problema
DentaCare je informacioni sistem za stomatološke ambulante koji omogućava efikasnu komunikaciju između pacijenata i stomatologa, kao i upravljanje terminima. Cilj sistema je da omogući:

Jednostavno zakazivanje stomatoloških pregleda i tretmana
Komunikaciju između pacijenata i stomatologa
Upload i pregled RTG snimaka
Edukaciju pacijenata kroz biblioteku stomatoloških saveta

Uloge Korisnika
Neulogovani Korisnici

Pregled edukativnih sadržaja o oralnoj higijeni
Pregled osnovnih informacija o stomatološkim procedurama
Pregled liste stomatologa i ambulanti (bez rasporeda termina)

Pacijenti (Ulogovani)

Registracija (ime, prezime, email, telefon, datum rođenja)
Pregled dostupnih stomatologa i ambulanti
Zakazivanje termina kod stomatologa (30/60 min slotovi)
Pregled kalendara svojih zakazanih termina
Chat sa stomatolozima (tekst, slike)
Upload RTG snimaka
Notifikacije i reminderi (dan/sat pre termina)
Pregled i preuzimanje PDF potvrda o zakazanim terminima
Ocenjivanje i pisanje recenzija stomatologa

Stomatolozi (Ulogovani)

Registracija sa informacijama o ambulanti i specijalizaciji
Profil sa specijalizacijom, slikom, certifikatima i radnim vremenom
Upravljanje rasporedom dostupnih termina
Pregled zakazanih termina
Chat sa pacijentima
Pregled upload-ovanih RTG snimaka od strane pacijenata
Generisanje PDF izveštaja/potvrda o zakazanim terminima
Pregled ocena i recenzija od strane pacijenata

Administratori

Upravljanje korisnicima (blokiranje, brisanje naloga)
Upravljanje edukativnim sadržajem
Pregled stomatologa i ambulanti
Odobravanje recenzija
Pregled statistika sistema (broj korisnika, termina, ambulanti)

Arhitektura Sistema
Mikroservisi (Rust)
Sistem se sastoji od 4 glavna mikroservisa:
1. auth_service (Autentifikacija i Autorizacija)

Registracija i login korisnika (pacijenti, stomatolozi, administratori)
JWT autentifikacija
Upravljanje ulogama i permissions
Refresh token mehanizam
Profili korisnika (osnovne informacije, slike)
Baza: PostgreSQL (korisnici, uloge, sesije, profili)

2. appointment_service (Upravljanje Terminima)

CRUD operacije nad terminima
Raspoređivanje slobodnih slotova (30/60 min)
Upravljanje kalendarom stomatologa
Provera dostupnosti i sprečavanje duplih rezervacija
Upravljanje ambulantama (informacije, adresa, radno vreme)
Baza: PostgreSQL (termini, rasporedi, dostupnost, ambulante)

3. media_service (Upravljanje Medijskim Fajlovima)

Upload RTG snimaka
Upload slika (profilne slike, slike za chat)
Skladištenje i preuzimanje fajlova
Generisanje PDF potvrda o zakazanim terminima
Validacija i kompresija slika
Baza: PostgreSQL (metadata o fajlovima - putanja, tip, vlasnik, datum)
Storage: AWS S3 / MinIO / Lokalni file system

4. chat_notification_service (Chat i Notifikacije)

Real-time chat između pacijenata i stomatologa
Sistem notifikacija i reminders
Push notifikacije
Email notifikacije (potvrda termina, reminderi)
WebSocket komunikacija za real-time poruke
Slanje slika u chat-u
Baza: MongoDB (poruke, chat istorija, notifikacije)

API Gateway

Centralna tačka pristupa za sve mikroservise
Rutiranje zahteva ka odgovarajućim servisima
Load balancing
Rate limiting
Authentication middleware (validacija JWT tokena)
CORS konfiguracija
Request/Response logging
File upload proxy (za velike fajlove)

Tehničke Specifikacije
Backend

Jezik: Rust
Framework: Axum / Actix-web
Autentifikacija: JWT tokens (access + refresh)
Komunikacija: REST API
API Gateway: Rust (Axum)
File Upload: Multipart form data handling

Baze Podataka

Relacione (PostgreSQL): auth_service, appointment_service, media_service
NoSQL (MongoDB): chat_notification_service
Deployment: Docker kontejneri za sve baze

Storage

RTG Snimci i Slike: AWS S3 / MinIO ili lokalni Docker volume
PDF Fajlovi: Privremeno generisani, dostupni za download

Frontend

Framework: Angular (najnovija verzija)
State Management: NgRx / Signals
Komunikacija: HttpClient sa REST API
Real-time: WebSocket za chat
UI Library: Angular Material / PrimeNG
File Upload: Angular file upload komponente
