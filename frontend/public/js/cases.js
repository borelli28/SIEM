import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;
const formId = 'cases-form';
let activeTab = 'comments';

function showAlert(message, type = 'error') {
    const alertContainer = document.getElementById('alert-container');
    if (!alertContainer) return;
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
        updateMainContent(caseData);
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

function updateMainContent(caseData) {
    const casesContainer = document.getElementById('cases-container');
    casesContainer.innerHTML = `
        <div class="case-details">
            <h2>${caseData.title}</h2>
            <p class="description">${caseData.description}</p>
            <div class="case-info">
                <p><strong>Created:</strong> ${new Date(caseData.created_at).toLocaleString()}</p>
                <p><strong>Last Updated:</strong> ${new Date(caseData.updated_at).toLocaleString()}</p>
            </div>
        </div>
    `;
}

function updateActiveTab(caseData) {
    const tabContent = document.getElementById('tab-content');
    if (!tabContent) return;

    switch(activeTab) {
        case 'comments':
            tabContent.innerHTML = `
                <div class="comments-section">
                    <ul class="comments-list">
                        ${caseData.comments.map(comment => `
                            <li class="comment">${comment}</li>
                        `).join('')}
                    </ul>
                </div>
            `;
            break;
        case 'observables':
            // Filter out alerts and logs from observables
            const otherObservables = caseData.observables.filter(obs => 
                obs.observable_type !== 'alert' && obs.observable_type !== 'log'
            );

            tabContent.innerHTML = `
                <div class="observables-section">
                    <ul class="observables-list">
                        ${otherObservables.map(obs => `
                            <li class="observable">
                                <strong>${obs.observable_type}:</strong> 
                                <pre>${obs.value}</pre>
                            </li>
                        `).join('')}
                    </ul>
                </div>
            `;
            break;
        case 'events':
            // Filter for alerts and logs only
            const events = caseData.observables.filter(obs => 
                obs.observable_type === 'alert' || obs.observable_type === 'log'
            );

            tabContent.innerHTML = `
                <div class="events-section">
                    ${events.map(event => {
                        const eventData = JSON.parse(event.value);
                        if (event.observable_type === 'alert') {
                            return `
                                <div class="event alert-event">
                                    <h4>Alert</h4>
                                    <p><strong>Message:</strong> ${eventData.message}</p>
                                    <p><strong>Severity:</strong> ${eventData.severity}</p>
                                    <p><strong>Created:</strong> ${new Date(eventData.created_at).toLocaleString()}</p>
                                </div>
                            `;
                        } else { // log
                            return `
                                <div class="event log-event">
                                    <h4>Log Entry</h4>
                                    <p><strong>Source:</strong> ${eventData.device_product || 'Unknown'}</p>
                                    <p><strong>Severity:</strong> ${eventData.severity || 'Unknown'}</p>
                                    <pre>${JSON.stringify(eventData, null, 2)}</pre>
                                </div>
                            `;
                        }
                    }).join('')}
                </div>
            `;
            break;
    }
}

function switchTab(tabName) {
    activeTab = tabName;
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

    const urlParams = new URLSearchParams(window.location.search);
    const caseId = urlParams.get('id');
    if (caseId) {
        loadCaseDetails(caseId);
    }
}

document.addEventListener('DOMContentLoaded', async () => {
    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }
    await fetchCsrfToken();

    // Get case ID from URL
    const urlParams = new URLSearchParams(window.location.search);
    const caseId = urlParams.get('id');
    
    if (!caseId) {
        window.location.href = '/list-cases';
        return;
    }

    await loadCaseDetails(caseId);

    document.getElementById('logout-btn').addEventListener('click', async () => {
        const result = await logout();
        if (result.success) {
            window.location.href = '/login';
        } else {
            showAlert(result.message, 'error');
        }
    });

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