import { Routes } from '@angular/router';
import { authGuard } from './guards/auth.guard';
import { roleGuard } from './guards/role.guard';

export const routes: Routes = [
    { path: '', redirectTo: '/home', pathMatch: 'full' },
    {
        path: 'home',
        loadComponent: () =>
            import('./components/shared/home/home').then(m => m.Home)
    },
    {
        path: 'login',
        loadComponent: () =>
            import('./components/auth/login/login').then(m => m.Login)
    },
    {
        path: 'register',
        loadComponent: () =>
            import('./components/auth/register/register').then(m => m.Register)
    },
    {
        path: 'dentists',
        loadComponent: () =>
            import('./components/shared/dentists/dentists').then(m => m.Dentists)
    },
    {
        path: 'education',
        loadComponent: () =>
            import('./components/shared/education/education').then(m => m.Education)
    },
    {
        path: 'profile',
        canActivate: [authGuard],
        loadComponent: () =>
            import('./components/shared/profile/profile').then(m => m.Profile)
    },
    {
        path: 'patient',
        canActivate: [authGuard, roleGuard],
        data: { role: 'patient' },
        children: [
            {
                path: 'dashboard',
                loadComponent: () =>
                    import('./components/patient/dashboard/dashboard').then(m => m.Dashboard)
            },
            {
                path: 'xray',
                loadComponent: () =>
                    import('./components/patient/xray/xray').then(m => m.Xray)
            },
            {
                path: 'appointments',
                loadComponent: () =>
                    import('./components/patient/appointments/appointments').then(m => m.Appointments)
            },
            {
                path: 'book',
                loadComponent: () =>
                    import('./components/patient/book-appointment/book-appointment').then(m => m.BookAppointment)
            }
        ]
    },
    {
        path: 'dentist',
        canActivate: [authGuard, roleGuard],
        data: { role: 'dentist' },
        children: [
            {
                path: 'dashboard',
                loadComponent: () =>
                    import('./components/dentist/dashboard/dashboard').then(m => m.Dashboard)
            },
            {
                path: 'reviews',
                loadComponent: () =>
                    import('./components/dentist/reviews/reviews').then(m => m.DentistReviews)
            },
            {
                path: 'xray',
                loadComponent: () =>
                    import('./components/dentist/xray/xray').then(m => m.DentistXray)
            },
            {
                path: 'schedule',
                loadComponent: () =>
                    import('./components/dentist/schedule/schedule').then(m => m.Schedule)
            },
            {
                path: 'appointments',
                loadComponent: () =>
                    import('./components/dentist/appointments/appointments').then(m => m.Appointments)
            }
        ]
    },
    {
        path: 'chat',
        canActivate: [authGuard],
        loadComponent: () =>
            import('./components/chat/chat-window/chat-window').then(m => m.ChatWindow)
    },
    { path: '**', redirectTo: '/home' }
];