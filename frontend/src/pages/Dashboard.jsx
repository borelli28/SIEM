import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { getCsrfToken } from '../services/csrfService';
import { getAuthenticationStatus, checkAuth, user } from '../services/authService';
import Navbar from '../components/Navbar';
import Chart from 'chart.js/auto';
import '../styles/Dashboard.css';

const Dashboard = () => {
    const [error, setError] = useState('');
    const [logs, setLogs] = useState([]);
    const [alerts, setAlerts] = useState([]);
    const [cases, setCases] = useState([]);
    const [isLoading, setIsLoading] = useState(true);
    const navigate = useNavigate();
    const [csrfToken, setCsrfToken] = useState(null);

    // Chart refs
    const severityChartRef = useRef(null);
    const alertChartRef = useRef(null);
    const signatureChartRef = useRef(null);
    const deviceChartRef = useRef(null);

    const formId = 'dashboard-form';

    useEffect(() => {
        const initAuth = async () => {
            await checkAuth();
            if (!getAuthenticationStatus()) {
                navigate('/login');
                return;
            }
            fetchAllData();
        };

        initAuth();

        return () => {
            // Cleanup charts on unmount
            [severityChartRef, alertChartRef, signatureChartRef, deviceChartRef].forEach(ref => {
                if (ref.current) {
                    ref.current.destroy();
                }
            });
        };
    }, [navigate]);

    const fetchAllData = async () => {
        setIsLoading(true);
        try {
            await Promise.all([
                fetchLogs(),
                fetchAlerts(),
                fetchCases()
            ]);
        } catch (err) {
            console.log(err.message);
            setError('Failed to load dashboard data');
        } finally {
            setIsLoading(false);
        }
    };

    const fetchLogs = async () => {
        try {
            await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/log/all/${user}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch logs');
            }

            const logsData = await response.json();
            setLogs(logsData);
            renderSeverityChart(logsData);
            renderSignatureChart(logsData);
        } catch (err) {
            console.log(err.message);
            setError('Failed to load logs');
        }
    };

    const fetchAlerts = async () => {
        try {
            await getCsrfToken(formId);
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
            renderAlertChart(alertsData);
        } catch (err) {
            console.log(err.message);
            setError('Failed to load alerts');
        }
    };

    const fetchCases = async () => {
        try {
            await getCsrfToken(formId);
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
            renderDeviceProductChart(logs);
        } catch (err) {
            console.log(err.message);
            setError('Failed to load cases');
        }
    };

    const renderSeverityChart = (logs) => {
        const ctx = document.getElementById('severityChart').getContext('2d');
        const data = processLogsData(logs);

        severityChartRef.current = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: [...data.severityData.labels, ...data.timeData.labels],
                datasets: [{
                    label: 'Log Distribution',
                    data: [...data.severityData.data, ...data.timeData.data],
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.2)',
                        'rgba(255, 206, 86, 0.2)',
                        'rgba(75, 192, 192, 0.2)',
                        'rgba(54, 162, 235, 0.2)',
                        'rgba(153, 102, 255, 0.2)',
                        'rgba(255, 159, 64, 0.2)',
                        'rgba(201, 203, 207, 0.2)'
                    ],
                    borderColor: [
                        'rgba(255, 99, 132, 1)',
                        'rgba(255, 206, 86, 1)',
                        'rgba(75, 192, 192, 1)',
                        'rgba(54, 162, 235, 1)',
                        'rgba(153, 102, 255, 1)',
                        'rgba(255, 159, 64, 1)',
                        'rgba(201, 203, 207, 1)'
                    ],
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: {
                            stepSize: 1
                        }
                    }
                },
                plugins: {
                    title: {
                        display: true,
                        text: 'Severity and Time Distribution'
                    },
                    legend: {
                        display: false
                    }
                }
            }
        });
    };

    const processLogsData = (logs) => {
        const severityCounts = {
            'High': 0,
            'Medium': 0,
            'Low': 0
        };

        const timeBasedCounts = {
            'Last Hour': 0,
            'Last 6 Hours': 0,
            'Last 24 Hours': 0,
            'Older': 0
        };

        const now = new Date();

        logs.forEach(log => {
            const severity = log.severity || 'Low';
            if (severity in severityCounts) {
                severityCounts[severity]++;
            }

            const logDate = new Date(log.created_at);
            const hoursDiff = (now - logDate) / (1000 * 60 * 60);

            if (hoursDiff <= 1) {
                timeBasedCounts['Last Hour']++;
            } else if (hoursDiff <= 6) {
                timeBasedCounts['Last 6 Hours']++;
            } else if (hoursDiff <= 24) {
                timeBasedCounts['Last 24 Hours']++;
            } else {
                timeBasedCounts['Older']++;
            }
        });

        return {
            severityData: {
                labels: Object.keys(severityCounts),
                data: Object.values(severityCounts)
            },
            timeData: {
                labels: Object.keys(timeBasedCounts),
                data: Object.values(timeBasedCounts)
            }
        };
    };

    const renderAlertChart = (alerts) => {
        const ctx = document.getElementById('alertChart')?.getContext('2d');
        if (!ctx) return;
        
        alertChartRef.current = new Chart(ctx, {
            type: 'pie',
            data: {
                labels: ['High', 'Medium', 'Low'],
                datasets: [{
                    data: [
                        alerts.filter(a => a.severity === 'High').length,
                        alerts.filter(a => a.severity === 'Medium').length,
                        alerts.filter(a => a.severity === 'Low').length
                    ],
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.2)',
                        'rgba(255, 206, 86, 0.2)',
                        'rgba(75, 192, 192, 0.2)'
                    ],
                    borderColor: [
                        'rgba(255, 99, 132, 1)',
                        'rgba(255, 206, 86, 1)',
                        'rgba(75, 192, 192, 1)'
                    ]
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Alert Severity Distribution'
                    }
                }
            }
        });
    };

    const renderSignatureChart = (logs) => {
        const ctx = document.getElementById('signatureChart')?.getContext('2d');
        if (!ctx) return;
        
        const signatureCounts = {};
        logs.forEach(log => {
            if (log.signature_id) {
                signatureCounts[log.signature_id] = (signatureCounts[log.signature_id] || 0) + 1;
            }
        });

        signatureChartRef.current = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: Object.keys(signatureCounts),
                datasets: [{
                    label: 'Signature Distribution',
                    data: Object.values(signatureCounts),
                    backgroundColor: 'rgba(54, 162, 235, 0.2)',
                    borderColor: 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Log Signatures Distribution'
                    }
                }
            }
        });
    };

    const renderDeviceProductChart = (logs) => {
        const ctx = document.getElementById('deviceChart')?.getContext('2d');
        if (!ctx) return;
        
        const deviceCounts = {};
        logs.forEach(log => {
            if (log.device_product) {
                deviceCounts[log.device_product] = (deviceCounts[log.device_product] || 0) + 1;
            }
        });

        deviceChartRef.current = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: Object.keys(deviceCounts),
                datasets: [{
                    data: Object.values(deviceCounts),
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.2)',
                        'rgba(54, 162, 235, 0.2)',
                        'rgba(255, 206, 86, 0.2)',
                        'rgba(75, 192, 192, 0.2)'
                    ],
                    borderColor: [
                        'rgba(255, 99, 132, 1)',
                        'rgba(54, 162, 235, 1)',
                        'rgba(255, 206, 86, 1)',
                        'rgba(75, 192, 192, 1)'
                    ]
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'Device Product Distribution'
                    }
                }
            }
        });
    };

    return (
        <div className="container">
            <h1>SIEM Dashboard</h1>
            <Navbar />
            <main>
                {error && <div className="alert error">{error}</div>}
                {isLoading ? (
                    <div className="loading">Loading dashboard data...</div>
                ) : (
                    <div className="dashboard-grid">
                        <section className="graph-card">
                            <canvas id="severityChart"></canvas>
                        </section>
                        <section className="graph-card">
                            <canvas id="alertChart"></canvas>
                        </section>
                        <section className="graph-card">
                            <canvas id="signatureChart"></canvas>
                        </section>
                        <section className="graph-card">
                            <canvas id="deviceChart"></canvas>
                        </section>
                    </div>
                )}
            </main>
        </div>
    );
};

export default Dashboard;