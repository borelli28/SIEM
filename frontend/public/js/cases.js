import { getAuthenticationStatus, logout, checkAuth, user } from '../../services/authService.js';
import { getCsrfToken } from '../../services/csrfService.js';

let csrfToken;
const formId = 'cases-form';
let activeTab = 'comments';
let currentCase = null;

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

async function fetchUsers() {
    try {
        const response = await fetch(`http://localhost:4200/backend/account/${user}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (!response.ok) {
            throw new Error('Failed to fetch user');
        }

        const userData = await response.json();
        const assigneeSelect = document.getElementById('case-assignee');
        assigneeSelect.innerHTML = `<option value="${userData.id}">${userData.name}</option>`;
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to load user', 'error');
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
    currentCase = caseData;
    
    document.getElementById('case-assignee').value = caseData.analyst_assigned;
    document.getElementById('case-status').value = caseData.status;
    document.getElementById('case-severity').value = caseData.severity;
    document.getElementById('case-category').value = caseData.category;
}

function updateMainContent(caseData) {
    const casesContainer = document.getElementById('cases-container');
    casesContainer.innerHTML = `
        <div class="case-details">
            <h2 class="case-title" id="case-title-display">${caseData.title}</h2>
            <input type="text" 
                   class="editable-field case-title hidden" 
                   value="${caseData.title}" 
                   id="case-title-input">
            
            <p class="case-description" id="case-description-display">${caseData.description}</p>
            <textarea 
                class="editable-field case-description hidden" 
                id="case-description-input">${caseData.description}</textarea>
            
            <div class="case-info">
                <p><strong>Created:</strong> ${new Date(caseData.created_at).toLocaleString()}</p>
                <p><strong>Last Updated:</strong> ${new Date(caseData.updated_at).toLocaleString()}</p>
            </div>
        </div>
    `;

    // Add click handlers for title
    const titleDisplay = document.getElementById('case-title-display');
    const titleInput = document.getElementById('case-title-input');
    titleDisplay.addEventListener('click', () => {
        titleDisplay.classList.add('hidden');
        titleInput.classList.remove('hidden');
        titleInput.focus();
    });
    titleInput.addEventListener('blur', () => {
        titleDisplay.textContent = titleInput.value;
        titleInput.classList.add('hidden');
        titleDisplay.classList.remove('hidden');
        document.getElementById('save-changes').style.display = 'block';
    });
    titleInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            titleInput.blur();
        }
    });

    // Add click handlers for description
    const descDisplay = document.getElementById('case-description-display');
    const descInput = document.getElementById('case-description-input');
    descDisplay.addEventListener('click', () => {
        descDisplay.classList.add('hidden');
        descInput.classList.remove('hidden');
        descInput.focus();
    });
    descInput.addEventListener('blur', () => {
        descDisplay.textContent = descInput.value;
        descInput.classList.add('hidden');
        descDisplay.classList.remove('hidden');
        document.getElementById('save-changes').style.display = 'block';
    });
}

async function updateActiveTab(caseData) {
    const tabContent = document.getElementById('tab-content');
    if (!tabContent) return;

    switch(activeTab) {
        case 'comments':
            // Fetch comments for the case
            let comments = [];
            try {
                const response = await fetch(`http://localhost:4200/backend/case/${caseData.id}/comments`, {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId
                    },
                    credentials: 'include'
                });

                if (!response.ok) {
                    throw new Error('Failed to fetch comments');
                }

                comments = await response.json();
            } catch (error) {
                console.error('Error fetching comments:', error);
                showAlert('Failed to load comments', 'error');
            }

            tabContent.innerHTML = `
                <div class="comments-section">
                    <div class="add-comment-container">
                        <button id="show-comment-form" class="primary-btn">Add Comment</button>
                        <form id="add-comment-form" class="hidden">
                            <textarea id="comment-text" 
                                     placeholder="Enter your comment" 
                                     required></textarea>
                            <div class="comment-form-actions">
                                <button type="submit" class="primary-btn">Submit</button>
                                <button type="button" id="cancel-comment" class="secondary-btn">Cancel</button>
                            </div>
                        </form>
                    </div>
                    <ul class="comments-list">
                        ${comments.map(comment => `
                            <li class="comment">
                                <div class="comment-content" id="comment-display-${comment.id}" onclick="showCommentEdit('${comment.id}')">
                                    ${comment.comment}
                                </div>
                                <textarea 
                                    class="comment-edit hidden" 
                                    id="comment-edit-${comment.id}"
                                    onblur="saveCommentEdit('${comment.id}')"
                                    >${comment.comment}</textarea>
                                <div class="comment-metadata">
                                    <span class="comment-date">${new Date(comment.created_at).toLocaleString()}</span>
                                </div>
                            </li>
                        `).join('')}
                    </ul>
                </div>
            `;

            // Add event listeners for the comment form
            const commentForm = document.getElementById('add-comment-form');
            const showCommentBtn = document.getElementById('show-comment-form');
            const cancelCommentBtn = document.getElementById('cancel-comment');

            showCommentBtn.addEventListener('click', () => {
                commentForm.classList.remove('hidden');
                showCommentBtn.classList.add('hidden');
            });

            cancelCommentBtn.addEventListener('click', () => {
                commentForm.classList.add('hidden');
                showCommentBtn.classList.remove('hidden');
            });

            commentForm.addEventListener('submit', async (e) => {
                e.preventDefault();
                const commentText = document.getElementById('comment-text').value;

                try {
                    const response = await fetch(`http://localhost:4200/backend/case/${currentCase.id}/comment`, {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'X-Form-ID': formId
                        },
                        credentials: 'include',
                        body: JSON.stringify(commentText)
                    });

                    if (!response.ok) {
                        throw new Error('Failed to add comment');
                    }

                    showAlert('Comment added successfully', 'success');
                    document.getElementById('comment-text').value = '';
                    commentForm.classList.add('hidden');
                    showCommentBtn.classList.remove('hidden');
                    await loadCaseDetails(currentCase.id);
                } catch (error) {
                    console.error('Error:', error);
                    showAlert('Failed to add comment', 'error');
                }
            });
            break;
        case 'observables':
            const otherObservables = caseData.observables.filter(obs => 
                obs.observable_type !== 'alert' && obs.observable_type !== 'log'
            );

            tabContent.innerHTML = `
                <div class="observables-section">
                    <div class="add-observable-container">
                        <button id="show-observable-form" class="primary-btn">Add Observable</button>
                        <form id="add-observable-form" class="hidden">
                            <select id="observable-type" required>
                                <option value="">Select Type</option>
                                <option value="ip">IP Address</option>
                                <option value="domain">Domain</option>
                                <option value="hash">File Hash</option>
                                <option value="url">URL</option>
                                <option value="email">Email</option>
                            </select>
                            <input type="text" id="observable-value" placeholder="Enter value" required>
                            <button type="submit" class="primary-btn">Add</button>
                            <button type="button" id="cancel-observable" class="secondary-btn">Cancel</button>
                        </form>
                    </div>
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

            // Add event listeners for the new form
            const addForm = document.getElementById('add-observable-form');
            const showFormBtn = document.getElementById('show-observable-form');
            const cancelBtn = document.getElementById('cancel-observable');

            showFormBtn.addEventListener('click', () => {
                addForm.classList.remove('hidden');
                showFormBtn.classList.add('hidden');
            });

            cancelBtn.addEventListener('click', () => {
                addForm.classList.add('hidden');
                showFormBtn.classList.remove('hidden');
            });

            addForm.addEventListener('submit', async (e) => {
                e.preventDefault();
                const type = document.getElementById('observable-type').value;
                const value = document.getElementById('observable-value').value;

                try {
                    const response = await fetch(`http://localhost:4200/backend/case/${currentCase.id}/observable`, {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'X-Form-ID': formId
                        },
                        credentials: 'include',
                        body: JSON.stringify({
                            observable_type: type,
                            value: value
                        })
                    });

                    if (!response.ok) {
                        throw new Error('Failed to add observable');
                    }

                    showAlert('Observable added successfully', 'success');
                    await loadCaseDetails(currentCase.id);
                } catch (error) {
                    console.error('Error:', error);
                    showAlert('Failed to add observable', 'error');
                }
            });
            break;
        case 'events':
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
                        } else {
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

function showCommentEdit(commentId) {
    const displayElement = document.getElementById(`comment-display-${commentId}`);
    const editElement = document.getElementById(`comment-edit-${commentId}`);

    displayElement.classList.add('hidden');
    editElement.classList.remove('hidden');
    editElement.focus();
}

async function saveCommentEdit(commentId) {
    const displayElement = document.getElementById(`comment-display-${commentId}`);
    const editElement = document.getElementById(`comment-edit-${commentId}`);
    const newComment = editElement.value;

    try {
        const response = await fetch(`http://localhost:4200/backend/case/comment/${commentId}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include',
            body: JSON.stringify(newComment)
        });

        if (!response.ok) {
            throw new Error('Failed to update comment');
        }

        displayElement.textContent = newComment;
        displayElement.classList.remove('hidden');
        editElement.classList.add('hidden');
        showAlert('Comment updated successfully', 'success');
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to update comment', 'error');
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

async function saveChanges() {
    if (!currentCase) return;

    const updatedCase = {
        ...currentCase,
        title: document.getElementById('case-title-input').value,
        description: document.getElementById('case-description-input').value,
        analyst_assigned: document.getElementById('case-assignee').value,
        status: document.getElementById('case-status').value,
        severity: document.getElementById('case-severity').value,
        category: document.getElementById('case-category').value
    };

    try {
        const response = await fetch(`http://localhost:4200/backend/case/${currentCase.id}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include',
            body: JSON.stringify(updatedCase)
        });

        if (!response.ok) {
            throw new Error('Failed to update case');
        }

        showAlert('Case updated successfully', 'success');
        await loadCaseDetails(currentCase.id);
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to update case', 'error');
    }
}

document.addEventListener('DOMContentLoaded', async () => {
    await checkAuth();
    if (!getAuthenticationStatus()) {
        window.location.href = '/login';
        return;
    }
    await fetchCsrfToken();
    await fetchUsers();

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

    document.getElementById('save-changes').addEventListener('click', saveChanges);

    ['case-assignee', 'case-status', 'case-severity', 'case-category'].forEach(id => {
        document.getElementById(id).addEventListener('change', () => {
            document.getElementById('save-changes').style.display = 'block';
        });
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

// Make functions globally accessible
window.showCommentEdit = function(commentId) {
    const displayElement = document.getElementById(`comment-display-${commentId}`);
    const editElement = document.getElementById(`comment-edit-${commentId}`);
    
    displayElement.classList.add('hidden');
    editElement.classList.remove('hidden');
    editElement.focus();
}

window.saveCommentEdit = async function(commentId) {
    const displayElement = document.getElementById(`comment-display-${commentId}`);
    const editElement = document.getElementById(`comment-edit-${commentId}`);
    const newComment = editElement.value;

    try {
        const response = await fetch(`http://localhost:4200/backend/case/comment/${commentId}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'X-Form-ID': formId
            },
            credentials: 'include',
            body: JSON.stringify(newComment)
        });

        if (!response.ok) {
            throw new Error('Failed to update comment');
        }

        displayElement.textContent = newComment;
        displayElement.classList.remove('hidden');
        editElement.classList.add('hidden');
        showAlert('Comment updated successfully', 'success');
    } catch (error) {
        console.error('Error:', error);
        showAlert('Failed to update comment', 'error');
    }
}