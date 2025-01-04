import { getAuthenticationStatus, checkAuth, logout } from '../../services/authService.js';

document.addEventListener('DOMContentLoaded', async () => {
    const ctx = document.getElementById('logsChart').getContext('2d');

    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }

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
            window.location.href = '/login';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });
});