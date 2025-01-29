import React, { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { getAuthenticationStatus, checkAuth, user } from '../services/authService';
import { getCsrfToken } from '../services/csrfService';
import Navbar from '../components/Navbar';
import '../styles/CasesList.css';

const CasesList = () => {
    const [cases, setCases] = useState([]);
    const [accountsMap, setAccountsMap] = useState({});
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [isFormVisible, setIsFormVisible] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const navigate = useNavigate();
    const formId = 'cases-form';

    useEffect(() => {
        const initCases = async () => {
            await checkAuth();
            if (!getAuthenticationStatus()) {
                navigate('/login');
                return;
            }
            await fetchCases();
        };

        initCases();
    }, [navigate]);

    useEffect(() => {
        const fetchAnalysts = async () => {
            const analysts = {};
            for (const caseItem of cases) {
                if (caseItem.analyst_assigned && !analysts[caseItem.analyst_assigned]) {
                    try {
                        const response = await fetch(`http://localhost:4200/backend/account/${caseItem.analyst_assigned}`, {
                            method: 'GET',
                            headers: {
                                'Content-Type': 'application/json',
                                'X-Form-ID': formId
                            },
                            credentials: 'include'
                        });

                        if (response.ok) {
                            const account = await response.json();
                            analysts[caseItem.analyst_assigned] = account.name;
                        }
                    } catch (err) {
                        console.error('Error fetching analyst:', err);
                    }
                }
            }
            setAccountsMap(analysts);
        };

        if (cases.length > 0) {
            fetchAnalysts();
        }
    }, [cases]);

    const showAlert = (message, type = 'error') => {
        if (type === 'error') setError(message);
        else setSuccess(message);

        setTimeout(() => {
            setError('');
            setSuccess('');
        }, 5000);
    };

    const fetchCases = async () => {
        try {
            const csrfToken = await getCsrfToken(formId);

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

            const casesData = await response.json();
            console.log(casesData);
            setCases(casesData);
        } catch (err) {
            console.error('Error:', err);
            showAlert('Failed to fetch cases');
        } finally {
            setIsLoading(false);
        }
    };

    const handleCreateCase = async (e) => {
        e.preventDefault();

        try {
            const csrfToken = await getCsrfToken(formId);

            const accountResponse = await fetch(`http://localhost:4200/backend/account/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!accountResponse.ok) {
                throw new Error('Failed to fetch account details');
            }

            const accountData = await accountResponse.json();

            const formData = {
                title: e.target.title.value,
                severity: e.target.severity.value,
                category: e.target.category.value,
                analyst_assigned: accountData.name,
                status: "open",
                description: "",
                observables: "[]"
            };

            console.log(user);
            console.log(accountData);
            console.log(formData);

            const response = await fetch(`http://localhost:4200/backend/case/${user}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                body: JSON.stringify(formData),
                credentials: 'include'
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Failed to create new case');
            }

            showAlert('Case created successfully', 'success');
            e.target.reset();
            setIsFormVisible(false);
            fetchCases();
        } catch (err) {
            showAlert(err.message);
        }
    };

    const handleDeleteCase = async (caseId) => {
        if (!window.confirm('Are you sure you want to delete this case?')) {
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
            fetchCases();
        } catch (err) {
            showAlert('Failed to delete case');
        }
    };

    const handleSelectCase = async (caseId) => {
        const alertId = sessionStorage.getItem('pendingAlertId');
        if (!alertId) {
            showAlert('No alert selected');
            return;
        }

        try {
            const csrfToken = await getCsrfToken(formId);
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

            sessionStorage.removeItem('pendingAlertId');
            showAlert('Alert added to case successfully', 'success');
            navigate(`/cases?id=${caseId}`);
        } catch (err) {
            showAlert('Failed to add alert to case');
        }
    };

    return (
        <div className="container">
            <h1>Cases List</h1>
            <Navbar />
            
            {error && <div className="alert error">{error}</div>}
            {success && <div className="alert success">{success}</div>}

            <div id="cases-list">
                {isLoading ? (
                    <div>Loading cases...</div>
                ) : cases.length === 0 ? (
                    <div>No cases found</div>
                ) : (
                    <table>
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
                        <tbody>
                            {cases.map(caseItem => (
                                <tr key={caseItem.id}>
                                    <td>
                                        <Link to={`/cases?id=${caseItem.id}`}>
                                            {caseItem.title}
                                        </Link>
                                    </td>
                                    <td>{caseItem.status}</td>
                                    <td>{caseItem.severity}</td>
                                    <td>{caseItem.category}</td>
                                    <td>{accountsMap[caseItem.analyst_assigned] || 'Loading...'}</td>
                                    <td>{new Date(caseItem.created_at).toLocaleString()}</td>
                                    <td>
                                        <button
                                            className="delete-btn"
                                            onClick={() => handleDeleteCase(caseItem.id)}
                                        >
                                            Delete
                                        </button>
                                        {window.location.search.includes('selectCase=true') && (
                                            <button
                                                onClick={() => handleSelectCase(caseItem.id)}
                                            >
                                                Select
                                            </button>
                                        )}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>

            <button 
                id="show-form-btn" 
                className="primary-btn"
                onClick={() => setIsFormVisible(!isFormVisible)}
            >
                {isFormVisible ? 'Hide Form' : 'Create New Case'}
            </button>

            <div className={`form-container ${isFormVisible ? '' : 'hidden'}`}>
                <form onSubmit={handleCreateCase}>
                    <h2>Create New Case</h2>
                    <div className="form-group">
                        <label htmlFor="title">Title:</label>
                        <input type="text" id="title" name="title" required />
                    </div>
                    <div className="form-group">
                        <label htmlFor="severity">Severity:</label>
                        <select id="severity" name="severity" required>
                            <option value="low">Low</option>
                            <option value="medium">Medium</option>
                            <option value="high">High</option>
                            <option value="critical">Critical</option>
                        </select>
                    </div>
                    <div className="form-group">
                        <label htmlFor="category">Category:</label>
                        <select id="category" name="category" required>
                            <option value="malware">Malware</option>
                            <option value="phishing">Phishing</option>
                            <option value="intrusion">Intrusion</option>
                            <option value="data_leak">Data Leak</option>
                            <option value="other">Other</option>
                        </select>
                    </div>
                    <button type="submit" className="primary-btn">Create Case</button>
                </form>
            </div>
        </div>
    );
};

export default CasesList;