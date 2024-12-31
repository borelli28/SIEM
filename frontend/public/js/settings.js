import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

const formId = 'settings-form';

document.addEventListener('DOMContentLoaded', async () => {
    const alertContainer = document.getElementById('alert-container');

    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }

    await fetchCsrfToken();
    await populateHostList();

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login';
        } else {
            alertContainer.innerHTML = `<div class="alert error">${result.message}</div>`;
        }
    });

    document.getElementById('logForm').addEventListener('submit', uploadLogs);
    document.getElementById('hostForm').addEventListener('submit', addHost);
});

let csrfToken;

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken(formId);
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
    }
}

async function populateHostList() {
    try {
        const response = await fetch(`http://localhost:4200/backend/host/all/${user}`, {
            method: 'GET',
            headers: {
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (response.ok) {
            const hosts = await response.json();
            const hostSelect = document.getElementById('hostSelect');
            hosts.forEach(host => {
                const option = document.createElement('option');
                option.value = host.id;
                option.textContent = `${host.hostname} (${host.ip_address})`;
                hostSelect.appendChild(option);
            });
        } else {
            console.error('Failed to fetch hosts');
        }
    } catch (error) {
        console.error('Error fetching hosts:', error);
    }
}

async function uploadLogs(event) {
    event.preventDefault();

    const logFile = document.getElementById('logFile').files[0];
    const hostId = document.getElementById('hostSelect').value;
    const alertContainer = document.getElementById('alert-container');
    const accountId = user;
    const loadingSpinner = document.getElementById('loadingSpinner');

    if (!logFile || !hostId) {
        alertContainer.innerHTML = '<div class="alert error">Please select a file and a host.</div>';
        return;
    }

    // Show the spinner
    loadingSpinner.style.display = 'block';

    const formData = new FormData();
    formData.append('file', logFile);
    formData.append('host_id', hostId);
    formData.append('account_id', accountId);

    try {
        const response = await fetch(`http://localhost:4200/backend/log/import`, {
            method: 'POST',
            headers: {
                'X-Form-ID': formId
            },
            body: formData,
            credentials: 'include'
        });

        // Hide the spinner after the request completes
        loadingSpinner.style.display = 'none';

        if (response.ok) {
            alertContainer.innerHTML = '<div class="alert success">Logs uploaded successfully!</div>';
            document.getElementById('logForm').reset();
        } else {
            const error = await response.json();
            alertContainer.innerHTML = `<div class="alert error">${error.message}</div>`;
        }
    } catch (error) {
        // Hide the spinner if an error occurs
        loadingSpinner.style.display = 'none';
        console.error('Error uploading logs:', error);
        alertContainer.innerHTML = '<div class="alert error">An error occurred while uploading logs.</div>';
    }
}

async function addHost(event) {
    event.preventDefault();

    try {
        const hostName = document.getElementById('hostName').value;
        const hostIP = document.getElementById('hostIP').value;
        const accountId = user;
        const alertContainer = document.getElementById('alert-container');

        const response = await fetch(`http://localhost:4200/backend/host/${accountId}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            body: JSON.stringify({ id: '0', hostname: hostName, ip_address: hostIP, account_id: accountId }),
            credentials: 'include'
        });

        if (response.ok) {
            alertContainer.innerHTML = '<div class="alert success">Host added successfully!</div>';
            document.getElementById('hostForm').reset();
            await populateHostList(); // Refresh the host list
        } else {
            const error = await response.json();
            alertContainer.innerHTML = `<div class="alert error">${error.message}</div>`;
        }
    } catch (error) {
        console.error('Error adding host:', error);
        alertContainer.innerHTML = '<div class="alert error">An error occurred while adding the host.</div>';
    }
}