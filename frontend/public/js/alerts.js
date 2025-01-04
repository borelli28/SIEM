import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;
const formId = 'alerts-form';

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken(formId);
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
        showAlert('Error fetching CSRF token', 'error');
    }
}

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
            showAlert(result.message, 'error');
        }
    });

    async function fetchAlerts() {
        try {
            const response = await fetch(`http://localhost:4200/backend/alert/all/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch alerts');
            }

            const alerts = await response.json();
            displayAlerts(alerts);
        } catch (error) {
            console.error('Error fetching alerts:', error);
            showAlert('Failed to load alerts', 'error');
        }
    }

    function displayAlerts(alerts) {
        const alertsBody = document.getElementById('alerts-body');
        alertsBody.innerHTML = '';

        alerts.forEach(alert => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${alert.message}</td>
                <td>${alert.rule_id}</td>
                <td>${alert.severity}</td>
                <td>${new Date(alert.created_at).toLocaleString()}</td>
            `;
            alertsBody.appendChild(row);
        });
    }

    document.getElementById('severity-filter').addEventListener('change', (e) => {
        const severity = e.target.value;
        if (severity === 'all') {
            fetchAlerts();
        } else {
            filterAlertsBySeverity(severity);
        }
    });

    async function filterAlertsBySeverity(severity) {
        try {
            const response = await fetch(`http://localhost:4200/backend/alert/all/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch alerts');
            }

            const alerts = await response.json();
            const filteredAlerts = alerts.filter(alert => 
                alert.severity.toLowerCase() === severity.toLowerCase()
            );
            displayAlerts(filteredAlerts);
        } catch (error) {
            console.error('Error filtering alerts:', error);
        }
    }

    function showAlert(message, type = 'error') {
        const alertContainer = document.getElementById('alert-container');
        alertContainer.innerHTML = `<div class="alert ${type}">${message}</div>`;
        setTimeout(() => {
            alertContainer.innerHTML = '';
        }, 5000);
    }

    await fetchAlerts();
});