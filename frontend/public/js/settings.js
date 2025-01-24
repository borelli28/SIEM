import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';
import { parseYAML } from '../../services/yamlParser.js';

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

    // Set up Account ID copy button
    document.getElementById('accountIdCopyBtn').addEventListener('click', () => {
        const accountId = document.getElementById('accountId').textContent;
        navigator.clipboard.writeText(accountId)
            .then(() => showAlert('Account ID copied to clipboard!', 'success'))
            .catch(err => showAlert('Failed to copy', 'error'));
    });

    document.getElementById('yamlFile').addEventListener('change', handleYamlFileUpload);
});

function handleYamlFileUpload(event) {
    const file = event.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = function(e) {
        try {
            const yamlContent = e.target.result;
            const ruleData = parseYAML(yamlContent);
            console.log('Parsed YAML:', ruleData);

            // Basic fields
            document.getElementById('title').value = ruleData.title || '';
            document.getElementById('description').value = ruleData.description || '';
            document.getElementById('status').value = ruleData.status || 'experimental';
            document.getElementById('author').value = ruleData.author || '';
            
            // Convert object to array if needed for tags
            const tagsArray = Object.values(ruleData.tags || {});
            document.getElementById('tags').value = tagsArray.join(', ');
            
            // Convert object to array if needed for references
            const referencesArray = Object.values(ruleData.references || {});
            document.getElementById('references').value = referencesArray.join(', ');
            
            // Logsource
            document.getElementById('logsource_category').value = ruleData.logsource.category || '';
            document.getElementById('logsource_product').value = ruleData.logsource.product || '';
            document.getElementById('logsource_service').value = ruleData.logsource.service || '';

            // Detection
            document.getElementById('detection_selection_field').value = 'msg';
            document.getElementById('detection_selection_value').value = ruleData.detection.selection.msg;
            document.getElementById('detection_condition').value = ruleData.detection.condition;

            // Convert object to array if needed for fields
            const fieldsArray = Object.values(ruleData.fields || {});
            document.getElementById('fields').value = fieldsArray.join(', ');
            
            // Convert object to array if needed for falsepositives
            const falsePositivesArray = Object.values(ruleData.falsepositives || {});
            document.getElementById('falsepositives').value = falsePositivesArray.join(', ');
            
            document.getElementById('level').value = ruleData.level.charAt(0).toUpperCase() + ruleData.level.slice(1);
            document.getElementById('enabled').checked = true;

            showAlert('YAML file loaded successfully!', 'success');
        } catch (error) {
            console.error('Error parsing YAML:', error);
            showAlert('Error parsing YAML file', 'error');
        }
    };
    reader.readAsText(file);
}

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

            // Populate host select dropdown
            const hostSelect = document.getElementById('hostSelect');
            hostSelect.innerHTML = '';

            // Update Account ID display
            document.getElementById('accountId').textContent = user;

            // Populate hosts table in agent setup section
            const hostsTableBody = document.getElementById('hostsTableBody');
            hostsTableBody.innerHTML = '';

            if (hosts.length === 0) {
                // Handle empty hosts case
                hostSelect.innerHTML = `<option value="" disabled selected>Add a new host below</option>`;
                hostsTableBody.innerHTML = '<tr><td colspan="4">No hosts available</td></tr>';
            } else {
                // Add default option to select
                hostSelect.innerHTML = '<option value="" disabled selected>Select a host</option>';

                // Populate both select and table
                hosts.forEach(host => {
                    // Add to select dropdown
                    const option = document.createElement('option');
                    option.value = host.id;
                    option.textContent = `${host.hostname} (${host.ip_address})`;
                    hostSelect.appendChild(option);

                    // Add to hosts table
                    const row = document.createElement('tr');
                    row.innerHTML = `
                        <td>${host.hostname}</td>
                        <td>${host.id}</td>
                        <td>${host.ip_address}</td>
                        <td><button class="copy-btn" data-copy="${host.id}">Copy ID</button></td>
                    `;
                    hostsTableBody.appendChild(row);
                });
            }

            // Add click handlers for host ID copy buttons
            document.querySelectorAll('.copy-btn').forEach(button => {
                button.addEventListener('click', () => {
                    const textToCopy = button.getAttribute('data-copy');
                    navigator.clipboard.writeText(textToCopy)
                        .then(() => showAlert('Host ID copied to clipboard!', 'success'))
                        .catch(err => showAlert('Failed to copy', 'error'));
                });
            });
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
        const currentDate = new Date();
        // (Sigma rule format: YYYY/MM/DD)
        const sigmaDate = currentDate.toISOString().replace('T', ' ').split('.')[0];
        // (Database DATETIME format)
        const dbTimestamp = currentDate.toISOString().replace('T', ' ').split('.')[0];

        const formData = {
            id: crypto.randomUUID(),
            account_id: user,
            title: document.getElementById('title').value,
            status: document.getElementById('status').value,
            description: document.getElementById('description').value,
            references: document.getElementById('references').value.split(',').map(s => s.trim()).filter(Boolean),
            tags: document.getElementById('tags').value.split(',').map(s => s.trim()).filter(Boolean),
            author: document.getElementById('author').value,
            date: sigmaDate,
            logsource: {
                category: document.getElementById('logsource_category').value,
                product: document.getElementById('logsource_product').value,
                service: document.getElementById('logsource_service').value
            },
            detection: {
                selection: {
                    [document.getElementById('detection_selection_field').value]: 
                        document.getElementById('detection_selection_value').value
                },
                condition: document.getElementById('detection_condition').value
            },
            fields: document.getElementById('fields').value.split(',').map(s => s.trim()).filter(Boolean),
            falsepositives: document.getElementById('falsepositives').value.split(',').map(s => s.trim()).filter(Boolean),
            level: document.getElementById('level').value,
            enabled: document.getElementById('enabled').checked,
            created_at: dbTimestamp,
            updated_at: dbTimestamp
        };

        // Validate required fields
        if (!formData.title || !formData.description || 
            !document.getElementById('detection_selection_field').value || 
            !document.getElementById('detection_selection_value').value) {
            showAlert('Please fill in all required fields', 'error');
            return;
        }

        const response = await fetch(`http://localhost:4200/backend/rule/${user}`, {
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