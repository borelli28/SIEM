import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { getAuthenticationStatus, checkAuth, user } from '../services/authService';
import { getCsrfToken } from '../services/csrfService';
import Navbar from '../components/Navbar';
import '../styles/Search.css';

const Search = () => {
    const [logs, setLogs] = useState([]);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [selectedLogId, setSelectedLogId] = useState(null);
    const [cases, setCases] = useState([]);
    const navigate = useNavigate();
    const formId = 'search-log-form';

    useEffect(() => {
        const initSearch = async () => {
            await checkAuth();
            if (!getAuthenticationStatus()) {
                navigate('/login');
                return;
            }

            // Set default time range
            const endDate = new Date();
            const startDate = new Date();
            startDate.setFullYear(endDate.getFullYear() - 2); // 2 years ago

            document.getElementById('startTime').value = startDate.toISOString().slice(0, 16);
            document.getElementById('endTime').value = endDate.toISOString().slice(0, 16);
        };

        initSearch();
    }, [navigate]);

    const showAlert = (message, type = 'error') => {
        if (type === 'error') setError(message);
        else setSuccess(message);

        setTimeout(() => {
            setError('');
            setSuccess('');
        }, 5000);
    };

    const handleSearch = async (e) => {
        e.preventDefault();
        const eqlQuery = document.getElementById('eqlQuery').value.trim();
        const startTime = document.getElementById('startTime').value
            ? document.getElementById('startTime').value + ':00.000-05:00'
            : document.getElementById('startTime').defaultValue + ':00.000-05:00';
        const endTime = document.getElementById('endTime').value
            ? document.getElementById('endTime').value + ':00.000-05:00'
            : document.getElementById('endTime').defaultValue + ':00.000-05:00';

        if (!eqlQuery) {
            showAlert('Please enter a search query');
            return;
        }

        await fetchFilteredLogs(eqlQuery, startTime, endTime);
    };

    const fetchFilteredLogs = async (query, startTime, endTime) => {
        try {
            const params = new URLSearchParams({
                query: query,
                account_id: user,
                start_time: startTime || '',
                end_time: endTime || ''
            });

            const response = await fetch(`http://localhost:4200/backend/log/filter?${params}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Failed to fetch logs');
            }

            const logsData = await response.json();
            setLogs(logsData);
        } catch (err) {
            console.error('Error:', err);
            showAlert(err.message);
        }
    };

    const addLogAsEvent = async (logId) => {
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

            const casesData = await response.json();
            setCases(casesData);
            setSelectedLogId(logId);
            setIsModalOpen(true);
        } catch (err) {
            console.error('Error:', err);
            showAlert('Failed to fetch cases');
        }
    };

    const handleCaseSelection = async (caseId) => {
        try {
            const addResponse = await fetch(`http://localhost:4200/backend/case/${caseId}/observable`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include',
                body: JSON.stringify({
                    observable_type: 'log',
                    value: selectedLogId
                })
            });

            if (!addResponse.ok) {
                throw new Error('Failed to add log as event');
            }

            showAlert('Log added as event successfully', 'success');
            setIsModalOpen(false);
        } catch (err) {
            showAlert('Failed to add log as event');
        }
    };

    const Modal = () => (
        <div className="modal">
            <div className="modal-content">
                <h3>Select Case</h3>
                <select id="case-select">
                    {cases.map(c => (
                        <option key={c.id} value={c.id}>{c.title}</option>
                    ))}
                </select>
                <div className="modal-actions">
                    <button 
                        onClick={() => handleCaseSelection(document.getElementById('case-select').value)}
                        className="primary-btn"
                    >
                        Add
                    </button>
                    <button 
                        onClick={() => setIsModalOpen(false)}
                        className="secondary-btn"
                    >
                        Cancel
                    </button>
                </div>
            </div>
        </div>
    );

    return (
        <div className="container">
            <h1>SIEM Log Search</h1>
            <Navbar />
            
            {error && <div className="alert error">{error}</div>}
            {success && <div className="alert success">{success}</div>}

            <form onSubmit={handleSearch}>
                <div className="search-controls">
                    <div className="time-range">
                        <div className="time-input">
                            <label htmlFor="startTime">From:</label>
                            <input 
                                type="datetime-local" 
                                id="startTime" 
                                name="startTime"
                            />
                        </div>
                        <div className="time-input">
                            <label htmlFor="endTime">To:</label>
                            <input 
                                type="datetime-local" 
                                id="endTime" 
                                name="endTime"
                            />
                        </div>
                    </div>
                    <div className="eql-input-container">
                        <textarea 
                            id="eqlQuery" 
                            placeholder='Enter EQL query (e.g., severity = "high" AND device_vendor = "cisco")'
                            rows="3"
                        ></textarea>
                        <div className="eql-help">
                            <h4>EQL Query Examples:</h4>
                            <ul>
                                <li>severity = "high" AND device_vendor = "cisco"</li>
                                <li>signature_id = "500" OR severity = "critical"</li>
                            </ul>
                        </div>
                    </div>
                    <button type="submit">Search</button>
                </div>
            </form>

            <div id="logs-count" className="logs-count">
                {logs.length > 0 && `Found ${logs.length} log${logs.length !== 1 ? 's' : ''}`}
            </div>
            
            <div className="logs-container">
                {logs.length === 0 ? (
                    'No logs found'
                ) : (
                    logs.map(log => (
                        <div key={log.id} className="log-entry">
                            <div className="log-content">
                                <pre>{JSON.stringify(log, null, 2)}</pre>
                            </div>
                            <div className="log-actions">
                                <button 
                                    className="add-event-btn"
                                    onClick={() => addLogAsEvent(log.id)}
                                >
                                    Add as Event
                                </button>
                            </div>
                        </div>
                    ))
                )}
            </div>

            {isModalOpen && <Modal />}
        </div>
    );
};

export default Search;