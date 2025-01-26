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

async function fetchFilteredLogs(query, startTime, endTime) {
    try {
        const params = new URLSearchParams({
            query: query,
            account_id: user,
            start_time: startTime || '',
            end_time: endTime || ''
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

window.handleSearch = async function(event) {
    event.preventDefault();
    const eqlQuery = document.getElementById('eqlQuery').value.trim();
    const startTime = document.getElementById('startTime').value;
    const endTime = document.getElementById('endTime').value;

    if (!eqlQuery) {
        showAlert('Please enter a search query', 'error');
        return;
    }

    await fetchFilteredLogs(eqlQuery, startTime, endTime);
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

    logs.forEach(log => {
        const logDiv = document.createElement('div');
        logDiv.className = 'log-entry';
        
        logDiv.innerHTML = `
            <div class="log-content">
                <pre>${JSON.stringify(log, null, 2)}</pre>
            </div>
            <div class="log-actions">
                <button class="add-event-btn" onclick="addLogAsEvent('${log.id}')">
                    Add as Event
                </button>
            </div>
        `;

        logsContainer.appendChild(logDiv);
    });
}

window.addLogAsEvent = async function(logId) {
    try {
        const response = await fetch(`http://localhost:4200/backend/case/all/${user}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to fetch cases');
        }

        const cases = await response.json();

        // Show case selection modal
        const caseId = await showCaseSelectionModal(cases);
        if (!caseId) return;

        // Add log as event to selected case
        const addResponse = await fetch(`http://localhost:4200/backend/case/${caseId}/observable`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include',
            body: JSON.stringify({
                observable_type: 'log',
                value: logId
            })
        });

        if (!addResponse.ok) {
            throw new Error('Failed to add log as event');
        }

        showAlert('Log added as event successfully', 'success');
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to add log as event', 'error');
    }
}

function showCaseSelectionModal(cases) {
    return new Promise((resolve) => {
        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-content">
                <h3>Select Case</h3>
                <select id="case-select">
                    ${cases.map(c => `
                        <option value="${c.id}">${c.title}</option>
                    `).join('')}
                </select>
                <div class="modal-actions">
                    <button onclick="submitCaseSelection()" class="primary-btn">Add</button>
                    <button onclick="closeCaseSelectionModal()" class="secondary-btn">Cancel</button>
                </div>
            </div>
        `;
        document.body.appendChild(modal);

        window.submitCaseSelection = () => {
            const selectedCase = document.getElementById('case-select').value;
            document.body.removeChild(modal);
            resolve(selectedCase);
        };

        window.closeCaseSelectionModal = () => {
            document.body.removeChild(modal);
            resolve(null);
        };
    });
}