import React, { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { getAuthenticationStatus, checkAuth, user } from '../services/authService';
import { getCsrfToken } from '../services/csrfService';
import Navbar from '../components/Navbar';
import '../styles/Cases.css';

const Cases = () => {
    const [activeTab, setActiveTab] = useState('comments');
    const [caseData, setCaseData] = useState(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [isLoading, setIsLoading] = useState(true);
    const [analysts, setAnalysts] = useState([]);
    const [showSaveButton, setShowSaveButton] = useState(false);
    const navigate = useNavigate();
    const location = useLocation();
    const formId = 'case-details-form';

    // Get case ID from URL query parameters
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
            await Promise.all([fetchCaseDetails(), fetchAnalysts()]);
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

    const fetchAnalysts = async () => {
        try {
            const response = await fetch('http://localhost:4200/backend/account/analysts', {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch analysts');
            }

            const data = await response.json();
            setAnalysts(data);
        } catch (err) {
            console.error('Error fetching analysts:', err);
        }
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
            setCaseData(data);
        } catch (err) {
            showAlert('Failed to fetch case details');
        } finally {
            setIsLoading(false);
        }
    };

    const handleSaveChanges = async () => {
        try {
            const csrfToken = await getCsrfToken(formId);
            const updatedData = {
                title: caseData.title,
                description: caseData.description,
                severity: document.getElementById('case-severity').value,
                status: document.getElementById('case-status').value,
                category: document.getElementById('case-category').value,
                analyst_assigned: document.getElementById('case-assignee').value
            };

            const response = await fetch(`http://localhost:4200/backend/case/${caseId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId,
                    'X-CSRF-Token': csrfToken
                },
                credentials: 'include',
                body: JSON.stringify(updatedData)
            });

            if (!response.ok) {
                throw new Error('Failed to update case');
            }

            showAlert('Case updated successfully', 'success');
            setShowSaveButton(false);
            await fetchCaseDetails();
        } catch (err) {
            showAlert('Failed to update case');
        }
    };

    const CommentsTab = () => {
        const [comments, setComments] = useState([]);
        const [showCommentForm, setShowCommentForm] = useState(false);

        useEffect(() => {
            fetchComments();
        }, []);

        const fetchComments = async () => {
            try {
                const response = await fetch(`http://localhost:4200/backend/case/${caseId}/comments`, {
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

                const data = await response.json();
                setComments(data);
            } catch (err) {
                showAlert('Failed to fetch comments');
            }
        };

        const handleAddComment = async (e) => {
            e.preventDefault();
            const commentText = e.target.comment.value;

            try {
                const csrfToken = await getCsrfToken(formId);
                const response = await fetch(`http://localhost:4200/backend/case/${caseId}/comment`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId,
                        'X-CSRF-Token': csrfToken
                    },
                    credentials: 'include',
                    body: JSON.stringify({ content: commentText })
                });

                if (!response.ok) {
                    throw new Error('Failed to add comment');
                }

                showAlert('Comment added successfully', 'success');
                e.target.reset();
                setShowCommentForm(false);
                await fetchComments();
            } catch (err) {
                showAlert('Failed to add comment');
            }
        };

        const handleDeleteComment = async (commentId) => {
            if (!window.confirm('Are you sure you want to delete this comment?')) {
                return;
            }

            try {
                const csrfToken = await getCsrfToken(formId);
                const response = await fetch(`http://localhost:4200/backend/case/comment/${commentId}`, {
                    method: 'DELETE',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId,
                        'X-CSRF-Token': csrfToken
                    },
                    credentials: 'include'
                });

                if (!response.ok) {
                    throw new Error('Failed to delete comment');
                }

                showAlert('Comment deleted successfully', 'success');
                await fetchComments();
            } catch (err) {
                showAlert('Failed to delete comment');
            }
        };

        return (
            <div className="comments-section">
                <div className="add-comment-container">
                    <button 
                        className="primary-btn"
                        onClick={() => setShowCommentForm(!showCommentForm)}
                    >
                        {showCommentForm ? 'Cancel' : 'Add Comment'}
                    </button>
                    
                    <form 
                        id="add-comment-form" 
                        className={showCommentForm ? '' : 'hidden'}
                        onSubmit={handleAddComment}
                    >
                        <textarea
                            id="comment-text"
                            name="comment"
                            placeholder="Enter your comment..."
                            required
                        ></textarea>
                        <div className="comment-form-actions">
                            <button type="submit" className="primary-btn">Add</button>
                        </div>
                    </form>
                </div>

                <div className="comments-list">
                    {comments.map(comment => (
                        <div key={comment.id} className="comment">
                            <div className="comment-header">
                                <div className="comment-content">
                                    {comment.content}
                                </div>
                                <button
                                    className="delete-comment-btn"
                                    onClick={() => handleDeleteComment(comment.id)}
                                >
                                    ×
                                </button>
                            </div>
                            <div className="comment-metadata">
                                Added by {comment.author} on {new Date(comment.created_at).toLocaleString()}
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        );
    };

    const ObservablesTab = () => {
        const [observables, setObservables] = useState([]);
        const [showObservableForm, setShowObservableForm] = useState(false);

        useEffect(() => {
            fetchObservables();
        }, []);

        const fetchObservables = async () => {
            try {
                const response = await fetch(`http://localhost:4200/backend/case/${caseId}/observables`, {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId
                    },
                    credentials: 'include'
                });

                if (!response.ok) {
                    throw new Error('Failed to fetch observables');
                }

                const data = await response.json();
                setObservables(data);
            } catch (err) {
                showAlert('Failed to fetch observables');
            }
        };

        const handleAddObservable = async (e) => {
            e.preventDefault();
            const formData = {
                observable_type: e.target.type.value,
                value: e.target.value.value
            };

            try {
                const csrfToken = await getCsrfToken(formId);
                const response = await fetch(`http://localhost:4200/backend/case/${caseId}/observable`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId,
                        'X-CSRF-Token': csrfToken
                    },
                    credentials: 'include',
                    body: JSON.stringify(formData)
                });

                if (!response.ok) {
                    throw new Error('Failed to add observable');
                }

                showAlert('Observable added successfully', 'success');
                e.target.reset();
                setShowObservableForm(false);
                await fetchObservables();
            } catch (err) {
                showAlert('Failed to add observable');
            }
        };

        const handleDeleteObservable = async (observableId) => {
            if (!window.confirm('Are you sure you want to delete this observable?')) {
                return;
            }

            try {
                const csrfToken = await getCsrfToken(formId);
                const response = await fetch(`http://localhost:4200/backend/case/observable/${observableId}`, {
                    method: 'DELETE',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId,
                        'X-CSRF-Token': csrfToken
                    },
                    credentials: 'include'
                });

                if (!response.ok) {
                    throw new Error('Failed to delete observable');
                }

                showAlert('Observable deleted successfully', 'success');
                await fetchObservables();
            } catch (err) {
                showAlert('Failed to delete observable');
            }
        };

        return (
            <div className="observables-section">
                <div className="add-observable-container">
                    <button 
                        className="primary-btn"
                        onClick={() => setShowObservableForm(!showObservableForm)}
                    >
                        {showObservableForm ? 'Cancel' : 'Add Observable'}
                    </button>
                    
                    <form 
                        id="add-observable-form"
                        className={showObservableForm ? '' : 'hidden'}
                        onSubmit={handleAddObservable}
                    >
                        <select name="type" required>
                            <option value="ip">IP Address</option>
                            <option value="domain">Domain</option>
                            <option value="hash">File Hash</option>
                            <option value="url">URL</option>
                        </select>
                        <input 
                            type="text" 
                            name="value" 
                            placeholder="Observable value"
                            required 
                        />
                        <div className="observable-form-actions">
                            <button type="submit" className="primary-btn">Add</button>
                        </div>
                    </form>
                </div>

                <div className="observables-list">
                    {observables.map(observable => (
                        <div key={observable.id} className="observable">
                            <div className="observable-header">
                                <div className="observable-content">
                                    <strong>{observable.type}:</strong> {observable.value}
                                </div>
                                <button
                                    className="delete-observable-btn"
                                    onClick={() => handleDeleteObservable(observable.id)}
                                >
                                    ×
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        );
    };

    const EventsTab = () => {
        const [events, setEvents] = useState([]);

        useEffect(() => {
            fetchEvents();
        }, []);

        const fetchEvents = async () => {
            try {
                const response = await fetch(`http://localhost:4200/backend/case/${caseId}/events`, {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Form-ID': formId
                    },
                    credentials: 'include'
                });

                if (!response.ok) {
                    throw new Error('Failed to fetch events');
                }

                const data = await response.json();
                setEvents(data);
            } catch (err) {
                showAlert('Failed to fetch events');
            }
        };

        return (
            <div className="events-section">
                {events.map(event => (
                    <div key={event.id} className={`event ${event.type}-event`}>
                        <h4>{event.type === 'alert' ? 'Alert Event' : 'Log Event'}</h4>
                        <pre>{JSON.stringify(JSON.parse(event.data), null, 2)}</pre>
                        <p>Added on {new Date(event.created_at).toLocaleString()}</p>
                    </div>
                ))}
            </div>
        );
    };

    return (
        <div className="container">
            <h1>Case Details</h1>
            <Navbar />
            
            {error && <div className="alert error">{error}</div>}
            {success && <div className="alert success">{success}</div>}

            {isLoading ? (
                <div>Loading case details...</div>
            ) : caseData ? (
                <>
                    <div className="case-header">
                        <h2>{caseData.title}</h2>
                    </div>

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
                        <div className="case-title">{caseData.title}</div>
                        <div className="case-description">{caseData.description}</div>
                    </div>

                    <div id="tab-content">
                        {activeTab === 'comments' && <CommentsTab />}
                        {activeTab === 'observables' && <ObservablesTab />}
                        {activeTab === 'events' && <EventsTab />}
                    </div>

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
                                        defaultValue={caseData.analyst_assigned}
                                        onChange={() => setShowSaveButton(true)}
                                    >
                                        {analysts.map(analyst => (
                                            <option key={analyst.id} value={analyst.id}>
                                                {analyst.name}
                                            </option>
                                        ))}
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
                                        id="case-category" 
                                        className="editable-field"
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
            ) : (
                <div>Case not found</div>
            )}
        </div>
    );
};

export default Cases;