import { isAuthenticated, checkAuth, logout } from './services/authService.js';

document.addEventListener('DOMContentLoaded', async () => {
    const alertContainer = document.getElementById('alert-container');
    const alertsBody = document.getElementById('alerts-body');
    const ctx = document.getElementById('logsChart').getContext('2d');

    let alerts = [
        { id: 1, ruleName: 'Suspicious Login', host: '192.168.1.100', severity: 'High' },
        { id: 2, ruleName: 'Failed SSH Attempts', host: '192.168.1.101', severity: 'Medium' },
        { id: 3, ruleName: 'Unusual Network Traffic', host: '192.168.1.102', severity: 'Low' },
    ];

    // Check Authentication
    await checkAuth();

    if (!isAuthenticated) {
        window.location.href = '/login.html';
        return;
    }

    // Load Alerts
    alerts.forEach(alert => {
        const row = document.createElement('tr');
        row.innerHTML = `<td>${alert.ruleName}</td><td>${alert.host}</td><td>${alert.severity}</td>`;
        alertsBody.appendChild(row);
    });

    // Chart
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: ['SSH', 'HTTP', 'FTP', 'DNS', 'SMTP'],
            datasets: [{
                label: '# of Logs',
                data: [12, 19, 3, 5, 2],
                backgroundColor: ['rgba(255, 99, 132, 0.2)', 'rgba(54, 162, 235, 0.2)', 'rgba(255, 206, 86, 0.2)', 'rgba(75, 192, 192, 0.2)', 'rgba(153, 102, 255, 0.2)'],
                borderColor: ['rgba(255, 99, 132, 1)', 'rgba(54, 162, 235, 1)', 'rgba(255, 206, 86, 1)', 'rgba(75, 192, 192, 1)', 'rgba(153, 102, 255, 1)'],
                borderWidth: 1
            }]
        },
        options: {
            scales: {
                y: {
                    beginAtZero: true
                }
            }
        }
    });

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login.html';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });
});