import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatExpansionModule } from '@angular/material/expansion';
import { RouterLink } from '@angular/router';

interface Article {
  title: string;
  icon: string;
  summary: string;
  content: string[];
  category: string;
}

@Component({
  selector: 'app-education',
  standalone: true,
  imports: [
    CommonModule, MatCardModule, MatIconModule,
    MatButtonModule, MatExpansionModule, RouterLink
  ],
  templateUrl: './education.html',
  styleUrl: './education.scss'
})
export class Education {
  articles: Article[] = [
    {
      title: 'Pravilno pranje zuba',
      icon: 'brush',
      category: 'Osnovna njega',
      summary: 'Naučite kako pravilno prati zube i koliko često.',
      content: [
        'Zube perite najmanje dva puta dnevno — ujutro i navečer prije spavanja.',
        'Koristite mekanu četkicu i fluoridnu pastu za zube.',
        'Perite zube kružnim pokretima po 2 minute.',
        'Ne zaboravite da operete jezik — na njemu se nakupljaju bakterije.',
        'Mijenjajte četkicu svaka 3 mjeseca ili kada se dlačice istrošu.'
      ]
    },
    {
      title: 'Upotreba zubnog konca',
      icon: 'healing',
      category: 'Osnovna njega',
      summary: 'Zubni konac uklanja naslage između zuba koje četkica ne može doseći.',
      content: [
        'Koristite zubni konac najmanje jednom dnevno, idealno navečer.',
        'Odlomite oko 45 cm konca i omotajte ga oko prstiju.',
        'Nježno provedite konac između zuba kliznim pokretima.',
        'Zaobljite konac oko svakog zuba i pokrenite gore-dolje.',
        'Nikad nemojte naglo povlačiti konac — može oštetiti desni.'
      ]
    },
    {
      title: 'Ishrana i zdravlje zuba',
      icon: 'restaurant',
      category: 'Prevencija',
      summary: 'Šta jesti, a čega se kloniti za zdrave zube i desni.',
      content: [
        'Ograničite unos šećera i zaslađenih pića — šećer hrani bakterije koje izazivaju karijes.',
        'Jedite hranu bogatu kalcijumom: mlijeko, sir, jogurt, brokoli.',
        'Vitamini C i D su ključni za zdravlje desni i kostiju.',
        'Pijte vodu umjesto sokova — voda ispire ostatke hrane.',
        'Izbjegavajte žvakanje leda i tvrde slatkiše — mogu slomiti zube.'
      ]
    },
    {
      title: 'Karijes — uzroci i prevencija',
      icon: 'warning',
      category: 'Bolesti',
      summary: 'Razumijevanje karijesa i kako ga spriječiti.',
      content: [
        'Karijes nastaje kada bakterije u ustima razgrađuju šećere i proizvode kiseline.',
        'Kiseline razaraju caklinu — vanjski zaštitni sloj zuba.',
        'Rani simptomi: osjetljivost na slatko, hladno ili toplo.',
        'Prevencija: redovno pranje, fluorid, manje šećera, posjete stomatologu.',
        'Neliječeni karijes može dovesti do bola, infekcije i gubitka zuba.'
      ]
    },
    {
      title: 'Parodontalna bolest',
      icon: 'local_hospital',
      category: 'Bolesti',
      summary: 'Bolest desni — tihi neprijatelj zdravlja zuba.',
      content: [
        'Parodontalna bolest je infekcija tkiva koje drže zube na mjestu.',
        'Uzrokovana je bakterijama u zubnom plaku koji se nakuplja uz desni.',
        'Simptomi: otečene, crvene ili krvareće desni, loš zadah.',
        'Napredna forma može dovesti do gubitka zuba i kostiju vilice.',
        'Liječi se profesionalnim čišćenjem, antibioticima i u težim slučajevima operacijom.'
      ]
    },
    {
      title: 'Koliko često posjećivati stomatologa',
      icon: 'event',
      category: 'Savjeti',
      summary: 'Redovne kontrole su ključ dugoročnog zdravlja zuba.',
      content: [
        'Preporučuje se posjeta stomatologu svakih 6 mjeseci.',
        'Djeca bi trebala početi posjećivati stomatologa od prvog zuba.',
        'Na kontroli stomatolog provjerava karijes, desni i opšte stanje usne šupljine.',
        'Profesionalno čišćenje uklanja kamenac koji se ne može ukloniti četkicom.',
        'Rano otkrivanje problema štedi novac, vrijeme i bol.'
      ]
    },
    {
      title: 'Ortodontski tretmani',
      icon: 'straighten',
      category: 'Tretmani',
      summary: 'Sve što trebate znati o aparatićima i ravnanju zuba.',
      content: [
        'Ortodoncija ispravlja nepravilno poređane zube i probleme s grizom.',
        'Fiksni aparati (breketi) su najčešći — nose se 1-3 godine.',
        'Mobilni aparati i providne šine (Invisalign) su diskretna alternativa.',
        'Ortodontski tretman je moguć u svakoj dobi, ali najlakši kod djece.',
        'Nakon skidanja aparata obavezno se nose retajneri da bi se sačuvao rezultat.'
      ]
    },
    {
      title: 'Izbjeljivanje zuba',
      icon: 'star',
      category: 'Estetika',
      summary: 'Bezbjedno izbjeljivanje zuba — šta funkcioniše, a šta ne.',
      content: [
        'Profesionalno izbjeljivanje kod stomatologa daje najsigurnije i najefikasnije rezultate.',
        'Kućni kiti za izbjeljivanje su slabiji ali mogu biti efikasni uz konzistentnu upotrebu.',
        'Izbjeljivanje ne djeluje na krunice, plombe i keramičke radove.',
        'Moguća privremena osjetljivost zuba tokom i nakon tretmana.',
        'Efekti traju 1-3 godine ovisno o prehrani i higijeni.'
      ]
    }
  ];

  categories = ['Sve', 'Osnovna njega', 'Prevencija', 'Bolesti', 'Savjeti', 'Tretmani', 'Estetika'];
  selectedCategory = 'Sve';

  get filteredArticles(): Article[] {
    if (this.selectedCategory === 'Sve') return this.articles;
    return this.articles.filter(a => a.category === this.selectedCategory);
  }

  selectCategory(cat: string): void {
    this.selectedCategory = cat;
  }
}