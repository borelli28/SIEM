import { getCsrfToken } from '../../services/csrfService.js';

async function handleSubmit(event) {
    event.preventDefault();
    const name = document.getElementById('name').value;
    const password = document.getElementById('password').value;
    const confirmPassword = document.getElementById('confirmPassword').value;
    const formId = 'register-form'

    const alertContainer = document.getElementById('alert-container');

    if (password !== confirmPassword) {
        alertContainer.innerHTML = `<div class="alert error">Passwords do not match!</div>`;
        return;
    }

    const newAccount = { id: '0', name: name, password: password, role: 'Admin'};

    try {
        const csrfToken = await getCsrfToken(formId);

        const response = await fetch('http://localhost:4200/backend/account/', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId,
                'X-XSRF-TOKEN': csrfToken
            },
            body: JSON.stringify(newAccount),
            credentials: 'include'
        });

        if (response.ok) {
            alertContainer.innerHTML = `<div class="alert success">Account created successfully!</div>`;
            document.getElementById('name').value = '';
            document.getElementById('password').value = '';
            document.getElementById('confirmPassword').value = '';
        } else {
            const errorData = await response.json();
            console.log(errorData);
            alertContainer.innerHTML = `<div class="alert error">Error: ${errorData.message || 'Failed to create account.'}</div>`;
        }
    } catch (error) {
        alertContainer.innerHTML = `<div class="alert error">Error creating account: ${error.message}</div>`;
    }
}