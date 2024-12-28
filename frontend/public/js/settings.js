import { getAuthenticationStatus, logout, checkAuth } from '../../services/authService.js';

document.addEventListener('DOMContentLoaded', async () => {
    const alertContainer = document.getElementById('alert-container');

    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });
});

async function addLogSource(event) {
    event.preventDefault();

    const sourceName = document.getElementById('sourceName').value;
    const sourceType = document.getElementById('sourceType').value;
    const sourceAddress = document.getElementById('sourceAddress').value;
    const alertContainer = document.getElementById('alert-container');

    // Send the new log source data to your backend (mock implementation)
    alertContainer.innerHTML = `<div class="alert success">Log Source "${sourceName}" added successfully!</div>`;

    document.getElementById('logSourceForm').reset();
}

async function addHost(event) {
    event.preventDefault();

    const hostName = document.getElementById('hostName').value;
    const hostIP = document.getElementById('hostIP').value;
    const alertContainer = document.getElementById('alert-container');

    // Send the new host data to your backend (mock implementation)
    alertContainer.innerHTML = `<div class="alert success">Host "${hostName}" with IP "${hostIP}" added successfully!</div>`;

    document.getElementById('hostForm').reset();
}