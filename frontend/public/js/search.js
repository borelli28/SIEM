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
    const logsContainer = document.getElementById('logs-container');
    const logsCountElement = document.getElementById('logs-count');
    logsContainer.innerHTML = '';
    const logsLength = logs.length;

    logsCountElement.textContent = `Found ${logsLength} log${logsLength !== 1 ? 's' : ''}`;

    if (!logs || logsLength === 0) {
        logsContainer.innerHTML = 'No logs found';
        return;
    }

    const preElement = document.createElement('pre');
    preElement.textContent = JSON.stringify(logs, null, 2);
    logsContainer.appendChild(preElement);
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