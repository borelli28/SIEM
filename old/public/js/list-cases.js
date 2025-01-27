import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;
const formId = 'cases-form';

function showAlert(message, type = 'error') {
    const alertContainer = document.getElementById('alert-container');
    if (!alertContainer) {
        console.error('Alert container not found');
        return;
    }
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

async function fetchAnalystName(analystId) {
    try {
        const response = await fetch(`http://localhost:4200/backend/account/${analystId}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to fetch analyst details');
        }

        const analyst = await response.json();
        return analyst.name;
    } catch (error) {
        console.error('Error fetching analyst:', error);
        return 'Unknown';
    }
}

async function fetchCases() {
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
        await displayCases(cases);
    } catch (error) {
        console.error('Error:', error);
        showAlert(error.message, 'error');
    }
}

async function displayCases(cases) {
    const casesContainer = document.getElementById('cases-list');
    casesContainer.innerHTML = '';

    if (!cases || cases.length === 0) {
        casesContainer.innerHTML = 'No cases found';
        return;
    }

    const table = document.createElement('table');
    table.innerHTML = `
        <thead>
            <tr>
                <th>Title</th>
                <th>Status</th>
                <th>Severity</th>
                <th>Category</th>
                <th>Assignee</th>
                <th>Created</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody></tbody>
    `;

    const tbody = table.querySelector('tbody');
    casesContainer.appendChild(table);

    // Get analyst names
    const casesWithAnalysts = await Promise.all(cases.map(async caseItem => {
        const analystName = await fetchAnalystName(caseItem.analyst_assigned);
        return { ...caseItem, analystName };
    }));

    const urlParams = new URLSearchParams(window.location.search);
    const selectCase = urlParams.get('selectCase');

    casesWithAnalysts.forEach(caseItem => {
        const row = document.createElement('tr');
        let actionsHtml = `<button class="delete-btn" onclick="deleteCase('${caseItem.id}')">Delete</button>`;
        
        if (selectCase === 'true') {
            actionsHtml += `<button onclick="selectCase('${caseItem.id}')">Select</button>`;
        }

        row.innerHTML = `
            <td><a href="/cases?id=${caseItem.id}">${caseItem.title}</a></td>
            <td>${caseItem.status}</td>
            <td>${caseItem.severity}</td>
            <td>${caseItem.category}</td>
            <td>${caseItem.analystName}</td>
            <td>${new Date(caseItem.created_at).toLocaleString()}</td>
            <td>${actionsHtml}</td>
        `;
        tbody.appendChild(row);
    });
}

window.selectCase = async function(caseId) {
    const alertId = sessionStorage.getItem('pendingAlertId');
    if (!alertId) {
        showAlert('No alert selected', 'error');
        return;
    }

    try {
        const alertResponse = await fetch(`http://localhost:4200/backend/alert/${alertId}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!alertResponse.ok) {
            throw new Error('Failed to fetch alert details');
        }

        const alertData = await alertResponse.json();

        // Add alert as observable
        await fetch(`http://localhost:4200/backend/case/${caseId}/observable`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include',
            body: JSON.stringify({
                observable_type: 'alert',
                value: JSON.stringify({
                    alert_id: alertId,
                    message: alertData.message,
                    severity: alertData.severity,
                    created_at: alertData.created_at
                })
            })
        });

        // Clear the stored alert ID
        sessionStorage.removeItem('pendingAlertId');
        
        showAlert('Alert added to case successfully', 'success');
        window.location.href = `/cases?id=${caseId}`;
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to add alert to case', 'error');
    }
};

async function createNewCase(event) {
    event.preventDefault();
    
    const formData = {
        title: document.getElementById('title').value,
        severity: document.getElementById('severity').value,
        category: document.getElementById('category').value,
        analyst_assigned: user,
        status: "open"
    };

    try {
        const response = await fetch(`http://localhost:4200/backend/case/${user}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include',
            body: JSON.stringify(formData)
        });

        if (!response.ok) {
            throw new Error('Failed to create new case');
        }

        // Reset form
        document.getElementById('create-case-form').reset();
        
        await fetchCases();
        showAlert('Case created successfully', 'success');
    } catch (error) {
        console.error('Error:', error);
        showAlert(error.message, 'error');
    }
}

window.deleteCase = async function(caseId) {
    if (!confirm('Are you sure you want to delete this case?')) {
        return;
    }

    try {
        const response = await fetch(`http://localhost:4200/backend/case/${caseId}`, {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to delete case');
        }

        showAlert('Case deleted successfully', 'success');
        await fetchCases();
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to delete case', 'error');
    }
};

document.addEventListener('DOMContentLoaded', async () => {
    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }
    await fetchCsrfToken();
    await fetchCases();

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login';
        } else {
            showAlert(result.message, 'error');
        }
    });

    const showFormBtn = document.getElementById('show-form-btn');
    const formContainer = document.querySelector('.form-container');
    
    showFormBtn.addEventListener('click', () => {
        formContainer.classList.toggle('hidden');
        showFormBtn.textContent = formContainer.classList.contains('hidden') 
            ? 'Create New Case' 
            : 'Hide Form';
    });

    document.getElementById('create-case-form').addEventListener('submit', async (event) => {
        await createNewCase(event);
        formContainer.classList.add('hidden');
        showFormBtn.textContent = 'Create New Case';
    });
});