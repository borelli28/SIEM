import { getAuthenticationStatus, checkAuth, logout, user } from '../../services/authService.js';

document.addEventListener('DOMContentLoaded', async () => {
    const ctx = document.getElementById('logsChart').getContext('2d');
    const alertContainer = document.getElementById('alert-container');

    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }

    async function fetchLogs() {
        try {
            const response = await fetch(`http://localhost:4200/backend/log/all/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch logs');
            }

            const logs = await response.json();
            return logs;
        } catch (error) {
            console.error('Error fetching logs:', error);
            alertContainer.innerHTML = '<div class="alert error">Failed to load logs</div>';
            return [];
        }
    }

    function processLogsData(logs) {
        const severityCounts = {
            'High': 0,
            'Medium': 0,
            'Low': 0
        };

        const timeBasedCounts = {
            'Last Hour': 0,
            'Last 6 Hours': 0,
            'Last 24 Hours': 0,
            'Older': 0
        };

        const now = new Date();

        logs.forEach(log => {
            // Count by severity
            const severity = log.severity || 'Low';
            if (severity in severityCounts) {
                severityCounts[severity]++;
            }

            // Count by time
            const logDate = new Date(log.created_at);
            const hoursDiff = (now - logDate) / (1000 * 60 * 60);

            if (hoursDiff <= 1) {
                timeBasedCounts['Last Hour']++;
            } else if (hoursDiff <= 6) {
                timeBasedCounts['Last 6 Hours']++;
            } else if (hoursDiff <= 24) {
                timeBasedCounts['Last 24 Hours']++;
            } else {
                timeBasedCounts['Older']++;
            }
        });

        return {
            severityData: {
                labels: Object.keys(severityCounts),
                data: Object.values(severityCounts)
            },
            timeData: {
                labels: Object.keys(timeBasedCounts),
                data: Object.values(timeBasedCounts)
            }
        };
    }

    const logs = await fetchLogs();
    const chartData = processLogsData(logs);

    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: [...chartData.severityData.labels, ...chartData.timeData.labels],
            datasets: [{
                label: 'Log Distribution',
                data: [...chartData.severityData.data, ...chartData.timeData.data],
                backgroundColor: [
                    'rgba(255, 99, 132, 0.2)',  // High
                    'rgba(255, 206, 86, 0.2)',  // Medium
                    'rgba(75, 192, 192, 0.2)',  // Low
                    'rgba(54, 162, 235, 0.2)',  // Last Hour
                    'rgba(153, 102, 255, 0.2)', // Last 6 Hours
                    'rgba(255, 159, 64, 0.2)',  // Last 24 Hours
                    'rgba(201, 203, 207, 0.2)'  // Older
                ],
                borderColor: [
                    'rgba(255, 99, 132, 1)',
                    'rgba(255, 206, 86, 1)',
                    'rgba(75, 192, 192, 1)',
                    'rgba(54, 162, 235, 1)',
                    'rgba(153, 102, 255, 1)',
                    'rgba(255, 159, 64, 1)',
                    'rgba(201, 203, 207, 1)'
                ],
                borderWidth: 1
            }]
        },
        options: {
            responsive: true,
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        stepSize: 1
                    }
                }
            },
            plugins: {
                title: {
                    display: true,
                    text: 'Severity and Time Distribution'
                },
                legend: {
                    display: false
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