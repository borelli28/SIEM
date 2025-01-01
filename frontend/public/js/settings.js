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

    // Event Listeners
    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login';
        } else {
            showAlert(result.message, 'error');
        }
    });

    document.getElementById('logForm').addEventListener('submit', uploadLogs);
    document.getElementById('hostForm').addEventListener('submit', addHost);
    document.getElementById('ruleForm').addEventListener('submit', createRule);
});

let csrfToken;

async function fetchCsrfToken() {
    try {
        csrfToken = await getCsrfToken(formId);
    } catch (error) {
        console.error('Error fetching CSRF token:', error);
        showAlert('Error fetching CSRF token', 'error');
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
            hostSelect.innerHTML = '';

            // No hosts
            if (hosts.length === 0) {
                const option = document.createElement('option');
                option.value = "";
                option.textContent = "Add a new host below";
                option.disabled = true;
                option.selected = true;
                hostSelect.appendChild(option);
            } else {
                const defaultOption = document.createElement('option');
                defaultOption.value = "";
                defaultOption.textContent = "Select a host";
                defaultOption.disabled = true;
                defaultOption.selected = true;
                hostSelect.appendChild(defaultOption);

                hosts.forEach(host => {
                    const option = document.createElement('option');
                    option.value = host.id;
                    option.textContent = `${host.hostname} (${host.ip_address})`;
                    hostSelect.appendChild(option);
                });
            }
        } else {
            showAlert('Failed to fetch hosts', 'error');
        }
    } catch (error) {
        console.error('Error fetching hosts:', error);
        showAlert('Error fetching hosts', 'error');
    }
}

async function uploadLogs(event) {
    event.preventDefault();

    const logFile = document.getElementById('logFile').files[0];
    const hostId = document.getElementById('hostSelect').value;
    const loadingSpinner = document.getElementById('loadingSpinner');

    if (!logFile || !hostId) {
        showAlert('Please select a file and a host', 'error');
        return;
    }

    loadingSpinner.style.display = 'block';

    const formData = new FormData();
    formData.append('file', logFile);
    formData.append('host_id', hostId);
    formData.append('account_id', user);

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
            showAlert('Logs uploaded successfully!', 'success');
            document.getElementById('logForm').reset();
        } else {
            const error = await response.json();
            showAlert(error.message, 'error');
        }
    } catch (error) {
        // Hide the spinner if an error occurs
        loadingSpinner.style.display = 'none';
        console.error('Error uploading logs:', error);
        showAlert('An error occurred while uploading logs', 'error');
    }
}

async function addHost(event) {
    event.preventDefault();

    const hostName = document.getElementById('hostName').value;
    const hostIP = document.getElementById('hostIP').value;

    if (!hostName || !hostIP) {
        showAlert('Please fill in all fields', 'error');
        return;
    }

    if (!isValidIpAddress(hostIP)) {
        showAlert('Please enter a valid IP address', 'error');
        return;
    }

    try {
        const response = await fetch(`http://localhost:4200/backend/host/${user}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            body: JSON.stringify({ 
                id: '0', 
                hostname: hostName, 
                ip_address: hostIP, 
                account_id: user 
            }),
            credentials: 'include'
        });

        if (response.ok) {
            showAlert('Host added successfully!', 'success');
            document.getElementById('hostForm').reset();
            await populateHostList();
        } else {
            const error = await response.json();
            showAlert(error.message, 'error');
        }
    } catch (error) {
        console.error('Error adding host:', error);
        showAlert('An error occurred while adding the host', 'error');
    }
}

async function createRule(event) {
    event.preventDefault();

    try {
        const formData = {
            id: crypto.randomUUID(),
            account_id: user,
            title: document.getElementById('title').value,
            status: document.getElementById('status').value,
            description: document.getElementById('description').value,
            references: document.getElementById('references').value.split(',').map(s => s.trim()).filter(Boolean),
            tags: document.getElementById('tags').value.split(',').map(s => s.trim()).filter(Boolean),
            author: document.getElementById('author').value,
            date: new Date().toISOString(),
            logsource: {
                category: document.getElementById('logsource_category').value,
                product: document.getElementById('logsource_product').value,
                service: document.getElementById('logsource_service').value
            },
            detection: {
                condition: document.getElementById('detection_condition').value
            },
            fields: document.getElementById('fields').value.split(',').map(s => s.trim()).filter(Boolean),
            falsepositives: document.getElementById('falsepositives').value.split(',').map(s => s.trim()).filter(Boolean),
            level: document.getElementById('level').value,
            enabled: document.getElementById('enabled').checked,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
        };

        // Validate required fields
        if (!formData.title || !formData.description || !formData.detection.condition) {
            showAlert('Please fill in all required fields', 'error');
            return;
        }

        const response = await fetch('http://localhost:4200/backend/rules', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            body: JSON.stringify(formData),
            credentials: 'include'
        });

        if (response.ok) {
            showAlert('Rule created successfully!', 'success');
            document.getElementById('ruleForm').reset();
        } else {
            const error = await response.json();
            showAlert(error.message, 'error');
        }
    } catch (error) {
        console.error('Error creating rule:', error);
        showAlert('An error occurred while creating the rule', 'error');
    }
}

// Helper Functions
function isValidIpAddress(ipAddress) {
    const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/;
    if (!ipRegex.test(ipAddress)) return false;
    
    const parts = ipAddress.split('.');
    return parts.every(part => {
        const num = parseInt(part, 10);
        return num >= 0 && num <= 255;
    });
}

function showAlert(message, type = 'error') {
    const alertContainer = document.getElementById('alert-container');
    alertContainer.innerHTML = `<div class="alert ${type}">${message}</div>`;
    
    // Auto-hide alert after 5 seconds
    setTimeout(() => {
        alertContainer.innerHTML = '';
    }, 5000);
}