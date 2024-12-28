import { getAuthenticationStatus, setAuthenticationStatus, logout } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken('login-form');
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
    }
}

async function handleLogin(event) {
    event.preventDefault();
    const name = document.getElementById('name').value;
    const password = document.getElementById('password').value;
    const alertContainer = document.getElementById('alert-container');
    const formId = 'login-form';

    const loginData = { id: "0", name: name, password: password, role: "no" };

    try {
        const response = await fetch('http://localhost:4200/backend/account/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            body: JSON.stringify(loginData),
            credentials: 'include'
        });

        if (response.ok) {
            const data = await response.json();
            setAuthenticationStatus(true);
            alertContainer.innerHTML = `<div class="alert success">Login successful! Redirecting...</div>`;
            setTimeout(() => window.location.href = '/', 500);
        } else {
            alertContainer.innerHTML = `<div class="alert error">Invalid credentials</div>`;
        }
    } catch (error) {
        console.error('Login error:', error);
        alertContainer.innerHTML = `<div class="alert error">An error occurred during login</div>`;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    fetchCsrfToken();
    const form = document.getElementById('login-form');
    form.addEventListener('submit', handleLogin);
});