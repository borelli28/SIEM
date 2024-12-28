import { logout } from '../../services/authService.js';

document.addEventListener('DOMContentLoaded', () => {
    const alertContainer = document.getElementById('alert-container');

    // Listen for logout button click
    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login.html';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });
});

// Function to handle search form submission
async function handleSearch(event) {
    event.preventDefault();
    const searchQuery = document.getElementById('searchQuery').value;
    const startDate = document.getElementById('startDate').value;
    const endDate = document.getElementById('endDate').value;
    const logType = document.getElementById('logType').value;
    const severity = document.getElementById('severity').value;
    const logsBody = document.getElementById('logs-body');

    // Here you would typically fetch the logs from the backend based on the search criteria
    logsBody.innerHTML = '<tr><td>No logs found based on your criteria</td></tr>'; // Mock implementation
    
    console.log('Searching with criteria:', { searchQuery, startDate, endDate, logType, severity });
}