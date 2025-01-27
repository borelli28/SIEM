import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { getAuthenticationStatus, checkAuth, user } from '../services/authService';
import { getCsrfToken } from '../services/csrfService';
import Navbar from '../components/Navbar';
import '../styles/Alerts.css';

const Alerts = () => {
    const [alerts, setAlerts] = useState([]);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const navigate = useNavigate();
    const formId = 'alerts-form';

    useEffect(() => {
        const initAlerts = async () => {
            await checkAuth();
            if (!getAuthenticationStatus()) {
                navigate('/login');
                return;
            }
            fetchAlerts();
        };

        initAlerts();
    }, [navigate]);

    const showAlert = (message, type = 'error') => {
        if (type === 'error') setError(message);
        else setSuccess(message);

        setTimeout(() => {
            setError('');
            setSuccess('');
        }, 5000);
    };

    const fetchAlerts = async () => {
        try {
            const response = await fetch(`http://localhost:4200/backend/alert/all/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch alerts');
            }

            const alertsData = await response.json();
            setAlerts(alertsData);
        } catch (err) {
            console.error('Error fetching alerts:', err);
            showAlert('Failed to load alerts');
        }
    };

    const checkAlertHasCase = async (alertId) => {
        try {
            const response = await fetch(`http://localhost:4200/backend/alert/has_case/${alertId}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to check alert case status');
            }

            const data = await response.json();
            return data.has_case;
        } catch (err) {
            console.error('Error checking alert case status:', err);
            return false;
        }
    };

    const acknowledgeAlert = async (alertId) => {
        try {
            const response = await fetch(`http://localhost:4200/backend/alert/acknowledge/${alertId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (response.ok) {
                showAlert('Alert acknowledged successfully', 'success');
                fetchAlerts();
            } else {
                throw new Error('Failed to acknowledge alert');
            }
        } catch (err) {
            showAlert('Failed to acknowledge alert');
        }
    };

    const deleteAlert = async (alertId) => {
        if (!window.confirm('Are you sure you want to delete this alert?')) {
            return;
        }

        try {
            const response = await fetch(`http://localhost:4200/backend/alert/${alertId}`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (response.ok) {
                showAlert('Alert deleted successfully', 'success');
                fetchAlerts();
            } else {
                throw new Error('Failed to delete alert');
            }
        } catch (err) {
            showAlert('Failed to delete alert');
        }
    };

    const addAlertToCase = (alertId) => {
        sessionStorage.setItem('pendingAlertId', alertId);
        navigate('/list-cases?selectCase=true');
    };

    const filterAlertsBySeverity = async (severity) => {
        if (severity === 'all') {
            fetchAlerts();
            return;
        }

        try {
            const response = await fetch(`http://localhost:4200/backend/alert/all/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch alerts');
            }

            const alertsData = await response.json();
            const filteredAlerts = alertsData.filter(alert => 
                alert.severity.toLowerCase() === severity.toLowerCase()
            );
            setAlerts(filteredAlerts);
        } catch (err) {
            console.error('Error filtering alerts:', err);
        }
    };

    return (
        <div className="container">
            <h1>SIEM Alerts</h1>
            <Navbar />
            
            {error && <div className="alert error">{error}</div>}
            {success && <div className="alert success">{success}</div>}

            <div className="filter-container">
                <label htmlFor="severity-filter">Filter by Severity:</label>
                <select 
                    id="severity-filter"
                    onChange={(e) => filterAlertsBySeverity(e.target.value)}
                >
                    <option value="all">All</option>
                    <option value="low">Low</option>
                    <option value="medium">Medium</option>
                    <option value="high">High</option>
                    <option value="critical">Critical</option>
                </select>
            </div>

            <table>
                <thead>
                    <tr>
                        <th>Message</th>
                        <th>Rule ID</th>
                        <th>Severity</th>
                        <th>Timestamp</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {alerts.map(alert => (
                        <tr key={alert.id}>
                            <td>{alert.message}</td>
                            <td>{alert.rule_id}</td>
                            <td>{alert.severity}</td>
                            <td>{new Date(alert.created_at).toLocaleString()}</td>
                            <td>
                                <button
                                    onClick={() => acknowledgeAlert(alert.id)}
                                    disabled={alert.acknowledged}
                                    className="acknowledge-btn"
                                >
                                    {alert.acknowledged ? 'Acknowledged' : 'Acknowledge'}
                                </button>
                                <button
                                    onClick={() => deleteAlert(alert.id)}
                                    className="delete-btn"
                                >
                                    Delete
                                </button>
                                <button
                                    onClick={() => addAlertToCase(alert.id)}
                                    disabled={alert.has_case}
                                    className="add-case-btn"
                                >
                                    {alert.has_case ? 'Added to Case' : 'Add to Case'}
                                </button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
};

export default Alerts;