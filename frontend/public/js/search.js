import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;
const formId = 'search-log-form';

function showAlert(message, type = 'error') {
    const alertContainer = document.getElementById('alert-container');
    alertContainer.innerHTML = `<div class="alert ${type}">${message}</div>`;
    setTimeout(() => {
        alertContainer.innerHTML = '';
    }, 5000);
}

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken(formId);
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
        showAlert('Error fetching CSRF token', 'error');
    }
}

document.addEventListener('DOMContentLoaded', async () => {
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
});

async function fetchFilteredLogs(query) {
    try {
        const params = new URLSearchParams({
            query: query,
            account_id: user
        });

        const response = await fetch(`http://localhost:4200/backend/log/filter?${params}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.error || 'Failed to fetch logs');
        }

        const logs = await response.json();
        displayLogs(logs);
    } catch (error) {
        console.error('Error:', error);
        showAlert(error.message, 'error');
    }
}

function displayLogs(logs) {
    const logsBody = document.getElementById('logs-body');
    logsBody.innerHTML = '';
    
    if (!logs || logs.length === 0) {
        logsBody.innerHTML = '<tr><td colspan="11">No logs found based on your criteria</td></tr>';
        return;
    }

    logs.forEach(log => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${log.id}</td>
            <td>${log.host_id}</td>
            <td>${log.version || 'N/A'}</td>
            <td>${log.device_vendor || 'N/A'}</td>
            <td>${log.device_product || 'N/A'}</td>
            <td>${log.device_version || 'N/A'}</td>
            <td>${log.signature_id || 'N/A'}</td>
            <td>${log.name || 'N/A'}</td>
            <td>${log.severity || 'N/A'}</td>
            <td>${log.extensions || 'N/A'}</td>
            <td>${new Date(log.created_at).toLocaleString()}</td>
        `;
        logsBody.appendChild(row);
    });
}

window.handleSearch = async function(event) {
    event.preventDefault();
    
    const eqlQuery = document.getElementById('eqlQuery').value.trim();
    
    if (!eqlQuery) {
        showAlert('Please enter a search query', 'error');
        return;
    }

    await fetchFilteredLogs(eqlQuery);
}