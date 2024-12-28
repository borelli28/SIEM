import { getCsrfToken } from '../../services/csrfService.js';

const formId = 'register-form'
let csrfToken;

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken(formId);
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
    }
}

async function handleSubmit(event) {
    event.preventDefault();
    const name = document.getElementById('name').value;
    const password = document.getElementById('password').value;
    const confirmPassword = document.getElementById('confirmPassword').value;
    const alertContainer = document.getElementById('alert-container');

    if (password !== confirmPassword) {
        alertContainer.innerHTML = `<div class="alert error">Passwords do not match!</div>`;
        return;
    }

    const newAccount = { id: '0', name: name, password: password, role: 'Admin'};

    try {
        const response = await fetch('http://localhost:4200/backend/account/', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            body: JSON.stringify(newAccount),
            credentials: 'include'
        });

        if (response.ok) {
            alertContainer.innerHTML = `<div class="alert success">Account created successfully!</div>`;
        } else {
            const errorData = await response.json();
            alertContainer.innerHTML = `<div class="alert error">Error: ${errorData.message || 'Failed to create account.'}</div>`;
        }
    } catch (error) {
        alertContainer.innerHTML = `<div class="alert error">Error creating account: ${error.message}</div>`;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    fetchCsrfToken();
    const form = document.getElementById('register-form');
    form.addEventListener('submit', handleSubmit);
});
