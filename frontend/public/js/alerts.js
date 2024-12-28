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

    // Function to fetch and display alerts (mock implementation)
    async function fetchAlerts() {
        // Here you would typically fetch the alerts from the backend
        const alertsBody = document.getElementById('alerts-body');
        alertsBody.innerHTML = ''; // Clear existing alerts
        
        // Mock data for alerts
        const alerts = [
            { ruleName: 'Suspicious Login', host: '192.168.1.100', severity: 'High', timestamp: new Date().toUTCString() },
            { ruleName: 'Failed SSH Attempts', host: '192.168.1.101', severity: 'Medium', timestamp: new Date().toUTCString() }
        ];

        alerts.forEach(alert => {
            const row = document.createElement('tr');
            row.innerHTML = `<td>${alert.ruleName}</td><td>${alert.host}</td><td>${alert.severity}</td><td>${alert.timestamp}</td>`;
            alertsBody.appendChild(row);
        });
    }

    // Call the function to load alerts on page load
    fetchAlerts();
});