import React, { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { getAuthenticationStatus, checkAuth, getCurrentUser } from '../services/authService';
import { getCsrfToken } from '../services/csrfService';
import Navbar from '../components/Navbar';
import CommentsTab from '../components/tabs/CommentsTab';
import ObservablesTab from '../components/tabs/ObservablesTab';
import EventsTab from '../components/tabs/EventsTab';
import '../styles/Cases.css';

const Cases = () => {
    const [activeTab, setActiveTab] = useState('comments');
    const [caseData, setCaseData] = useState(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [isLoading, setIsLoading] = useState(true);
    const [showSaveButton, setShowSaveButton] = useState(false);
    const [currentUser, setCurrentUser] = useState(null);
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [isEditingDescription, setIsEditingDescription] = useState(false);
    const navigate = useNavigate();
    const location = useLocation();
    const formId = 'case-details-form';
    const caseId = new URLSearchParams(location.search).get('id');

    useEffect(() => {
        const initCase = async () => {
            await checkAuth();
            if (!getAuthenticationStatus()) {
                navigate('/login');
                return;
            }
            if (!caseId) {
                navigate('/list-cases');
                return;
            }
            await Promise.all([fetchCaseDetails(), fetchCurrentUser()]);
        };

        initCase();
    }, [caseId, navigate]);

    const showAlert = (message, type = 'error') => {
        if (type === 'error') setError(message);
        else setSuccess(message);

        setTimeout(() => {
            setError('');
            setSuccess('');
        }, 5000);
    };

    const fetchCurrentUser = async () => {
        const userData = await getCurrentUser();
        if (!userData) {
            console.log("Current user is null");
            return;
        }
        setCurrentUser(userData);
    };

    const fetchCaseDetails = async () => {
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
                throw new Error('Failed to fetch case details');
            }

            const data = await response.json();
            if (!data) {
                throw new Error('Case not found');
            }
            setCaseData(data);
        } catch (err) {
            showAlert('Failed to fetch case details: ' + err.message);
            setCaseData(null);
        } finally {
            setIsLoading(false);
        }
    };

    const handleSaveChanges = async () => {
        try {
            const csrfToken = await getCsrfToken(formId);
            const newAnalyst = document.getElementById('case-assignee').value;

            const updatedData = {
                ...caseData,
                title: caseData.title,
                description: caseData.description,
                severity: document.getElementById('case-severity').value,
                status: document.getElementById('case-status').value,
                category: document.getElementById('case-category').value,
                analyst_assigned: newAnalyst || caseData.analyst_assigned
            };

            const response = await fetch(`http://localhost:4200/backend/case/${caseId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include',
                body: JSON.stringify(updatedData)
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to update case');
            }

            showAlert('Case updated successfully', 'success');
            setShowSaveButton(false);
            await fetchCaseDetails();
        } catch (err) {
            showAlert('Failed to update case: ' + err.message);
        }
    };

    return (
        <div className="cases-container">
            <h1>Case Details</h1>
            <Navbar />

            {error && <div className="alert error">{error}</div>}
            {success && <div className="alert success">{success}</div>}

            {isLoading ? (
                <div>Loading case details...</div>
            ) : !caseData ? (
                <div className="alert error">Case not found or failed to load</div>
            ) : (
                <>
                    <h2 className="case-header">{caseData.title}</h2>

                    <div className="tabs">
                        <button 
                            className={`tab-btn ${activeTab === 'comments' ? 'active' : ''}`}
                            onClick={() => setActiveTab('comments')}
                        >
                            COMMENTS
                        </button>
                        <button 
                            className={`tab-btn ${activeTab === 'observables' ? 'active' : ''}`}
                            onClick={() => setActiveTab('observables')}
                        >
                            OBSERVABLES
                        </button>
                        <button 
                            className={`tab-btn ${activeTab === 'events' ? 'active' : ''}`}
                            onClick={() => setActiveTab('events')}
                        >
                            EVENTS
                        </button>
                    </div>

                    <div id="cases-container">
                        <div className="case-title">
                            {isEditingTitle ? (
                                <input
                                    type="text"
                                    className="editable-field"
                                    value={caseData.title}
                                    onChange={(e) => {
                                        setCaseData({
                                            ...caseData,
                                            title: e.target.value
                                        });
                                        setShowSaveButton(true);
                                    }}
                                    onBlur={() => setIsEditingTitle(false)}
                                    autoFocus
                                />
                            ) : (
                                <div onClick={() => setIsEditingTitle(true)}>
                                    {caseData.title}
                                </div>
                            )}
                        </div>
                        <div className="case-description">
                            {isEditingDescription ? (
                                <textarea
                                    className="editable-field"
                                    value={caseData.description || ''}
                                    onChange={(e) => {
                                        setCaseData({
                                            ...caseData,
                                            description: e.target.value
                                        });
                                        setShowSaveButton(true);
                                    }}
                                    onBlur={() => setIsEditingDescription(false)}
                                    autoFocus
                                />
                            ) : (
                                <div onClick={() => setIsEditingDescription(true)}>
                                    {caseData.description || 'No description available'}
                                </div>
                            )}
                        </div>
                    </div>

                    {activeTab === 'comments' && 
                        <CommentsTab 
                            caseId={caseId} 
                            formId={formId} 
                            showAlert={showAlert} 
                        />
                    }
                    {activeTab === 'observables' && 
                        <ObservablesTab 
                            caseId={caseId} 
                            formId={formId} 
                            showAlert={showAlert} 
                        />
                    }
                    {activeTab === 'events' && 
                        <EventsTab 
                            caseId={caseId} 
                            formId={formId} 
                            showAlert={showAlert} 
                        />
                    }

                    <div className="case-details-sidebar">
                        <div className="summary-section">
                            <div className="section-header">
                                <h3>Summary</h3>
                                <span className="collapse-icon">^</span>
                            </div>
                            <div className="section-content">
                                <div className="detail-row">
                                    <span className="label">Assignee:</span>
                                    <select 
                                        id="case-assignee" 
                                        className="editable-field"
                                        value={caseData.analyst_assigned || ""}
                                        onChange={(e) => {
                                            setShowSaveButton(true);
                                            const newValue = e.target.value;
                                            setCaseData({
                                                ...caseData,
                                                analyst_assigned: newValue
                                            });
                                        }}
                                    >
                                        <option value="">Unassigned</option>
                                        {currentUser && (
                                            <option value={currentUser.id}>
                                                {currentUser.name}
                                            </option>
                                        )}
                                    </select>
                                </div>
                                <div className="detail-row">
                                    <span className="label">Status:</span>
                                    <select 
                                        id="case-status" 
                                        className="editable-field"
                                        defaultValue={caseData.status}
                                        onChange={() => setShowSaveButton(true)}
                                    >
                                        <option value="open">Open</option>
                                        <option value="in_progress">In Progress</option>
                                        <option value="closed">Closed</option>
                                        <option value="hold">Hold</option>
                                    </select>
                                </div>
                            </div>
                        </div>

                        <div className="details-section">
                            <div className="section-header">
                                <h3>Details</h3>
                                <span className="collapse-icon">^</span>
                            </div>
                            <div className="section-content">
                                <div className="detail-row">
                                    <span className="label">Severity:</span>
                                    <select 
                                        id="case-severity" 
                                        className="editable-field"
                                        defaultValue={caseData.severity}
                                        onChange={() => setShowSaveButton(true)}
                                    >
                                        <option value="low">Low</option>
                                        <option value="medium">Medium</option>
                                        <option value="high">High</option>
                                        <option value="critical">Critical</option>
                                    </select>
                                </div>
                                <div className="detail-row">
                                    <span className="label">Category:</span>
                                    <input 
                                        type="text" 
                                        className="editable-field"
                                        id="case-category"
                                        defaultValue={caseData.category}
                                        onChange={() => setShowSaveButton(true)}
                                    />
                                </div>
                            </div>
                        </div>
                        {showSaveButton && (
                            <button 
                                id="save-changes" 
                                className="primary-btn"
                                onClick={handleSaveChanges}
                            >
                                Save Changes
                            </button>
                        )}
                    </div>
                </>
            )}
        </div>
    );
};

export default Cases;
