import { logout } from '../../services/authService.js';

document.addEventListener('DOMContentLoaded', () => {
    const alertContainer = document.getElementById('alert-container');

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login.html';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });
});

// Function to add log source (mock implementation)
async function addLogSource(event) {
    event.preventDefault();

    const sourceName = document.getElementById('sourceName').value;
    const sourceType = document.getElementById('sourceType').value;
    const sourceAddress = document.getElementById('sourceAddress').value;
    const alertContainer = document.getElementById('alert-container');

    // Here you would send the new log source data to your backend
    alertContainer.innerHTML = `<div class="alert success">Log Source ${sourceName} added successfully!</div>`;
    // Clear the fields after submission
    document.getElementById('sourceName').value = '';
    document.getElementById('sourceAddress').value = '';
}