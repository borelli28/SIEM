import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { getAuthenticationStatus, checkAuth, user } from '../services/authService';
import { getCsrfToken } from '../services/csrfService';
import Navbar from '../components/Navbar';
import '../styles/Settings.css';
import { parseYAML } from '../services/yamlParser';

const Settings = () => {
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [hosts, setHosts] = useState([]);
    const [isLoading, setIsLoading] = useState(false);
    const navigate = useNavigate();
    const formId = 'settings-form';

    useEffect(() => {
        const initSettings = async () => {
            await checkAuth();
            if (!getAuthenticationStatus()) {
                navigate('/login');
                return;
            }
            await populateHostList();
        };

        initSettings();
    }, [navigate]);

    const showAlert = (message, type = 'error') => {
        if (type === 'error') setError(message);
        else setSuccess(message);

        setTimeout(() => {
            setError('');
            setSuccess('');
        }, 5000);
    };

    const populateHostList = async () => {
        try {
            await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/host/all/${user}`, {
                method: 'GET',
                headers: {
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (response.ok) {
                const hostsData = await response.json();
                setHosts(hostsData);
            } else {
                showAlert('Failed to fetch hosts', 'error');
            }
        } catch (err) {
            console.error('Error fetching hosts:', err);
            showAlert('Error fetching hosts', 'error');
        }
    };

    const handleCopyId = async (id) => {
        try {
            await navigator.clipboard.writeText(id);
            showAlert('ID copied to clipboard!', 'success');
        } catch (err) {
            showAlert('Failed to copy', 'error');
        }
    };

    const handleFileUpload = async (e) => {
        const file = e.target.files[0];
        const hostId = document.getElementById('hostSelect').value;

        if (!file || !hostId) {
            showAlert('Please select a file and a host', 'error');
            return;
        }

        setIsLoading(true);
        const formData = new FormData();
        formData.append('file', file);
        formData.append('host_id', hostId);
        formData.append('account_id', user);

        try {
            await getCsrfToken(formId);
            const response = await fetch('http://localhost:4200/backend/log/import', {
                method: 'POST',
                headers: {
                    'X-Form-ID': formId
                },
                body: formData,
                credentials: 'include'
            });

            if (response.ok) {
                showAlert('Logs uploaded successfully!', 'success');
                e.target.value = '';
            } else {
                const error = await response.json();
                showAlert(error.message, 'error');
            }
        } catch (err) {
            showAlert('An error occurred while uploading logs', 'error');
        } finally {
            setIsLoading(false);
        }
    };

    const handleNewHost = async (e) => {
        e.preventDefault();
        const hostname = e.target.hostname.value;
        const ipAddress = e.target.ipAddress.value;

        if (!hostname || !ipAddress) {
            showAlert('Please fill in all fields', 'error');
            return;
        }

        try {
            await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/host/${user}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                body: JSON.stringify({
                    id: '0',
                     account_id: user,
                    hostname: hostname,
                    ip_address: ipAddress
                }),
                credentials: 'include'
            });

            if (response.ok) {
                showAlert('Host added successfully!', 'success');
                e.target.reset();
                await populateHostList();
            } else {
                const error = await response.json();
                showAlert(error.message, 'error');
            }
        } catch (err) {
            showAlert('An error occurred while adding the host', 'error');
        }
    };

    const handleRuleUpload = async (e) => {
        const file = e.target.files[0];

        if (!file) {
            showAlert('Please select a file', 'error');
            return;
        }

        // Check if the file extension is .yaml or .yml
        const fileExtension = file.name.split('.').pop().toLowerCase();
        if (fileExtension !== 'yaml' && fileExtension !== 'yml') {
            showAlert('Please upload only .yaml or .yml files', 'error');
            e.target.value = '';
            return;
        }

        setIsLoading(true);

        try {
            const fileContent = await file.text();
            const parsedRule = parseYAML(fileContent);

            const currentDate = new Date();
            // Format for created_at and updated_at (RFC3339)
            const timestamp = currentDate.toISOString();
            // Format for rule date (YYYY-MM-DD HH:MM:SS)
            const ruleDate = currentDate.toISOString()
                .replace('T', ' ')
                .replace('Z', '')
                .split('.')[0];

            const ruleData = {
                id: '0',
                account_id: user,
                title: parsedRule.title,
                status: parsedRule.status,
                description: parsedRule.description,
                references: parsedRule.references?.undefined || [],
                tags: parsedRule.tags?.undefined || [],
                author: parsedRule.author,
                date: ruleDate,
                logsource: {
                    category: parsedRule.logsource.category,
                    product: parsedRule.logsource.product,
                    service: parsedRule.logsource.service
                },
                detection: {
                    selection: parsedRule.detection.selection,
                    condition: parsedRule.detection.condition
                },
                fields: parsedRule.fields?.undefined || [],
                falsepositives: parsedRule.falsepositives?.undefined || [],
                level: parsedRule.level.charAt(0).toUpperCase() + parsedRule.level.slice(1),
                enabled: true,
                created_at: timestamp,
                updated_at: timestamp
            };

            await getCsrfToken(formId);
            const response = await fetch('http://localhost:4200/backend/rule/import', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                body: JSON.stringify(ruleData),
                credentials: 'include'
            });

            if (response.ok) {
                showAlert('Rules uploaded successfully!', 'success');
                e.target.value = '';
            } else {
                const error = await response.json();
                showAlert(error.message, 'error');
            }
        } catch (err) {
            showAlert('An error occurred while uploading rules', 'error');
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="container">
            <h1>SIEM Settings</h1>
            <Navbar />
            
            {error && <div className="alert error">{error}</div>}
            {success && <div className="alert success">{success}</div>}

            <main>
                <section className="agent-info">
                    <div className="info-group">
                        <h3>Account ID</h3>
                        <div className="copy-container">
                            <span>{user}</span>
                            <button 
                                className="primary-btn" 
                                onClick={() => handleCopyId(user)}
                            >
                                Copy
                            </button>
                        </div>
                    </div>

                    <div className="info-group">
                        <h3>Available Hosts</h3>
                        <table id="hostsTable">
                            <thead>
                                <tr>
                                    <th>Host Name</th>
                                    <th>Host ID</th>
                                    <th>IP Address</th>
                                    <th>Action</th>
                                </tr>
                            </thead>
                            <tbody>
                                {hosts.length === 0 ? (
                                    <tr><td colSpan="4">No hosts available</td></tr>
                                ) : (
                                    hosts.map(host => (
                                        <tr key={host.id}>
                                            <td>{host.hostname}</td>
                                            <td>{host.id}</td>
                                            <td>{host.ip_address}</td>
                                            <td>
                                                <button 
                                                    className="primary-btn"
                                                    onClick={() => handleCopyId(host.id)}
                                                >
                                                    Copy ID
                                                </button>
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </section>

                <section className="form-sections">
                    <h2>Upload Logs</h2>
                    <form onSubmit={(e) => {
                        e.preventDefault();
                        const fileInput = e.target.querySelector('input[type="file"]');
                        handleFileUpload({ target: fileInput });
                    }}>
                        <div id="input-spinner-container">
                            <select id="hostSelect" required>
                                <option value="">Select Host</option>
                                {hosts.map(host => (
                                    <option key={host.id} value={host.id}>
                                        {host.hostname}
                                    </option>
                                ))}
                            </select>
                            <input 
                                type="file" 
                                accept=".log,.txt"
                                required
                            />
                            {isLoading && <div className="spinner"></div>}
                        </div>
                        <button type="submit" className="primary-btn">Upload Logs</button>
                    </form>
                </section>

                <section className="form-sections">
                    <h2>Add New Host</h2>
                    <form onSubmit={handleNewHost}>
                        <div className="form-group">
                            <input
                                type="text"
                                id="hostname"
                                name="hostname"
                                placeholder="Hostname"
                                required
                            />
                        </div>
                        <div className="form-group">
                            <input
                                type="text"
                                id="ipAddress"
                                name="ipAddress"
                                placeholder="IP Address"
                                required
                            />
                        </div>
                        <button type="submit" className="primary-btn">Add Host</button>
                    </form>
                </section>

                <section className="form-sections">
                    <h2>Add New Sigma Rule</h2>
                    <p>Only <b>YAML</b> files accepted</p>
                    <form onSubmit={(e) => {
                        e.preventDefault();
                        const fileInput = e.target.querySelector('input[type="file"]');
                        handleRuleUpload({ target: fileInput });
                    }}>
                        <div id="input-spinner-container">
                            <input 
                                type="file" 
                                accept=".yaml,.yml"
                                required
                            />
                            {isLoading && <div className="spinner"></div>}
                        </div>
                        <button type="submit" className="primary-btn">Upload Rule</button>
                    </form>
                </section>
            </main>
        </div>
    );
};

export default Settings;