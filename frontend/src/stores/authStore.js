import { writable } from 'svelte/store';

export const isAuthenticated = writable(false);
export const user = writable(null);

export async function checkAuth() {
    try {
        const response = await fetch('http://localhost:4200/backend/check-auth', {
            credentials: 'include'
        });
        if (response.ok) {
            const data = await response.json();
            isAuthenticated.set(true);
            user.set(data.user);
        } else {
            isAuthenticated.set(false);
            user.set(null);
        }
    } catch (error) {
        console.error('Error checking auth:', error);
        isAuthenticated.set(false);
        user.set(null);
    }
}

export async function logout() {
    try {
        await fetch('http://localhost:4200/backend/logout', {
            method: 'POST',
            credentials: 'include'
        });
        isAuthenticated.set(false);
        user.set(null);
    } catch (error) {
        console.error('Error logging out:', error);
    }
}