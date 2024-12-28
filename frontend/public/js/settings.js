import { getAuthenticationStatus, logout, checkAuth } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

document.addEventListener('DOMContentLoaded', async () => {
    const alertContainer = document.getElementById('alert-container');

    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }

    await fetchCsrfToken();

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });
});

let csrfToken;

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken('settings-form');
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
    }
}

async function addLogSource(event) {
    event.preventDefault();

    const sourceName = document.getElementById('sourceName').value;
    const sourceType = document.getElementById('sourceType').value;
    const sourceAddress = document.getElementById('sourceAddress').value;
    const accountId = user.id;
    const alertContainer = document.getElementById('alert-container');

    const response = await fetch(`http://localhost:4200/backend/log/${accountId}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Form-ID': 'logSourceForm'
        },
        body: JSON.stringify({ name: sourceName, type: sourceType, address: sourceAddress }),
        credentials: 'include'
    });

    if (response.ok) {
        const data = await response.json();
        alertContainer.innerHTML = `<div class="alert success">Log Source "${data.name}" added successfully!</div>`;
        document.getElementById('logSourceForm').reset();
    } else {
        const error = await response.json();
        alertContainer.innerHTML = `<div class="alert error">${error.message}</div>`;
    }
}

async function addHost(event) {
    event.preventDefault();

    const hostName = document.getElementById('hostName').value;
    const hostIP = document.getElementById('hostIP').value;
    const accountId = user.id;
    const alertContainer = document.getElementById('alert-container');

    const response = await fetch(`http://localhost:4200/backend/host/${accountId}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Form-ID': 'hostForm'
        },
        body: JSON.stringify({ name: hostName, ip: hostIP }),
        credentials: 'include'
    });

    if (response.ok) {
        const data = await response.json();
        alertContainer.innerHTML = `<div class="alert success">Host "${data.name}" with IP "${data.ip}" added successfully!</div>`;
        document.getElementById('hostForm').reset();
    } else {
        const error = await response.json();
        alertContainer.innerHTML = `<div class="alert error">${error.message}</div>`;
    }
}