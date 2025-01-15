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
        const response = await fetch('http://localhost:4200/backend/cases', {
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
            </div>
        `;
        casesContainer.appendChild(caseElement);
    });
}

function switchTab(tabName) {
    activeTab = tabName;
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
    // Implement tab content switching logic here
}

async function createNewCase() {
    // Implement new case creation logic
    try {
        const response = await fetch('http://localhost:4200/backend/cases', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId,
                'X-CSRF-Token': csrfToken
            },
            credentials: 'include',
            body: JSON.stringify({
                title: 'New Case',
                status: 'in progress',
                severity: 'medium'
            })
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

document.addEventListener('DOMContentLoaded', async () => {
    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }
    await fetchCsrfToken();
    await fetchCases();

    // Event Listeners
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

    // Initialize collapse/expand functionality for sidebar sections
    document.querySelectorAll('.collapse-icon').forEach(icon => {
        icon.addEventListener('click', (e) => {
            const content = e.target.closest('.section-header')
                .nextElementSibling;
            content.style.display = 
                content.style.display === 'none' ? 'block' : 'none';
        });
    });
});