import { isAuthenticated, logout } from '../../services/authService.js';

async function handleLogin(event) {
    event.preventDefault();
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    const alertContainer = document.getElementById('alert-container');

    const loginData = { name: username, password: password };

    try {
        const response = await fetch('http://localhost:4200/backend/account/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(loginData),
            credentials: 'include'
        });

        if (response.ok) {
            const data = await response.json();
            isAuthenticated = true;
            alertContainer.innerHTML = `<div class="alert success">Login successful! Redirecting...</div>`;
            setTimeout(() => window.location.href = '/', 500);
        } else {
            alertContainer.innerHTML = `<div class="alert error">Invalid credentials</div>`;
        }
    } catch (error) {
        alertContainer.innerHTML = `<div class="alert error">An error occurred during login</div>`;
    }
}