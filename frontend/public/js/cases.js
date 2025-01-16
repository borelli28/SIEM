import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;
const formId = 'cases-form';
let activeTab = 'comments';

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
        displayCases(cases);
    } catch (error) {
        console.error('Error:', error);
        showAlert(error.message, 'error');
    }
}

function displayCases(cases) {
    const casesContainer = document.getElementById('cases-container');
    casesContainer.innerHTML = '';

    if (!cases || cases.length === 0) {
        casesContainer.innerHTML = 'No cases found';
        return;
    }

    cases.forEach(caseItem => {
        const caseElement = document.createElement('div');
        caseElement.className = 'case-item';
        caseElement.innerHTML = `
            <h3>${caseItem.title}</h3>
            <p>${caseItem.description}</p>
            <div class="case-meta">
                <span>Status: ${caseItem.status}</span>
                <span>Severity: ${caseItem.severity}</span>
                <span>Category: ${caseItem.category}</span>
                <span>Assignee: ${caseItem.analyst_assigned}</span>
            </div>
            <div class="case-actions">
                <button class="delete-btn" onclick="event.stopPropagation(); deleteCase('${caseItem.id}')">Delete</button>
            </div>
        `;
        caseElement.addEventListener('click', () => loadCaseDetails(caseItem.id));
        casesContainer.appendChild(caseElement);
    });
}

async function loadCaseDetails(caseId) {
    try {
        const response = await fetch(`http://localhost:4200/backend/case/${caseId}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to load case details');
        }

        const caseData = await response.json();
        updateSidebar(caseData);
        updateActiveTab(caseData);
    } catch (error) {
        console.error('Error:', error);
        showAlert(error.message, 'error');
    }
}

function updateSidebar(caseData) {
    document.getElementById('case-assignee').textContent = caseData.analyst_assigned;
    document.getElementById('case-status').textContent = caseData.status;
    document.getElementById('case-severity').textContent = caseData.severity;
    document.getElementById('case-category').textContent = caseData.category;
}

function updateActiveTab(caseData) {
    const tabContent = document.getElementById('cases-container');

    switch(activeTab) {
        case 'comments':
            tabContent.innerHTML = '<ul>' + 
                caseData.comments.map(comment => `<li>${comment}</li>`).join('') +
                '</ul>';
            break;
        case 'observables':
            tabContent.innerHTML = '<ul>' + 
                caseData.observables.map(obs => 
                    `<li>${obs.observable_type}: ${obs.value}</li>`
                ).join('') +
                '</ul>';
            break;
        case 'events':
            tabContent.innerHTML = '<p>Events tab content</p>';
            break;
    }
}

async function createNewCase() {
    try {
        const response = await fetch(`http://localhost:4200/backend/case/${user}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId,
                'X-CSRF-Token': csrfToken
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

function switchTab(tabName) {
    activeTab = tabName;
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

    const selectedCase = document.querySelector('.case-item.selected');
    if (selectedCase) {
        loadCaseDetails(selectedCase.dataset.caseId);
    }
}

// Event listeners remain the same
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

    document.getElementById('new-case-btn').addEventListener('click', createNewCase);
    document.getElementById('refresh-btn').addEventListener('click', fetchCases);

    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            switchTab(e.target.dataset.tab);
        });
    });

    document.querySelectorAll('.collapse-icon').forEach(icon => {
        icon.addEventListener('click', (e) => {
            const content = e.target.closest('.section-header').nextElementSibling;
            content.style.display = content.style.display === 'none' ? 'block' : 'none';
        });
    });
});

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