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

    const casesWithAnalysts = await Promise.all(cases.map(async caseItem => {
        const analystName = await fetchAnalystName(caseItem.analyst_assigned);
        return { ...caseItem, analystName };
    }));

    casesWithAnalysts.forEach(caseItem => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td><a href="/cases?id=${caseItem.id}">${caseItem.title}</a></td>
            <td>${caseItem.status}</td>
            <td>${caseItem.severity}</td>
            <td>${caseItem.category}</td>
            <td>${caseItem.analystName}</td>
            <td>${new Date(caseItem.created_at).toLocaleString()}</td>
            <td>
                <button class="delete-btn" onclick="deleteCase('${caseItem.id}')">Delete</button>
            </td>
        `;
        tbody.appendChild(row);
    });
}

async function createNewCase() {
    try {
        const response = await fetch(`http://localhost:4200/backend/case/${user}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to create new case');
        }

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
});