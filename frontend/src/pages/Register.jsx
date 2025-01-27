import React, { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { getCsrfToken } from '../services/csrfService';
import '../styles/LogReg.css';

const Register = () => {
    const [formData, setFormData] = useState({
        name: '',
        password: '',
        confirmPassword: ''
    });
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const navigate = useNavigate();
    const [csrfToken, setCsrfToken] = useState(null);

    const formId = 'register-form'

    useEffect(() => {
        const fetchCsrfToken = async () => {
            try {
                const token = await getCsrfToken(formId);
                setCsrfToken(token);
            } catch (err) {
                setError('Failed to initialize form security');
            }
        };
        fetchCsrfToken();
    }, []);

    const handleChange = (e) => {
        setFormData({
            ...formData,
            [e.target.name]: e.target.value
        });
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        setError('');

        if (formData.password !== formData.confirmPassword) {
            setError('Passwords do not match!');
            return;
        }

        setIsLoading(true);

        const newAccount = {
            id: '0',
            name: formData.name,
            password: formData.password,
            role: 'Admin'
        };

        try {
            const response = await fetch('http://localhost:4200/backend/account/', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                body: JSON.stringify(newAccount),
                credentials: 'include'
            });

            const data = await response.json();

            if (response.status === 400) {
                if (data.status === "error") {
                    if (data.message.includes("Missing required field")) {
                        setError(data.message);
                    } else if (data.message === "Invalid role provided") {
                        setError("Invalid role provided");
                    } else {
                        setError(data.message);
                    }
                    return;
                }
            }

            if (!response.ok) {
                if (data.status === "error") {
                    setError(data.message); 
                } else {
                    setError('Registration failed');
                }
                return;
            }

            setSuccess("Success!");
            navigate('/login');
        } catch (err) {
            console.error('Error:', err);
            setError('Network error occurred. Please try again.');
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="auth-container">
            <div className="auth-box">
                <h2>Register</h2>
                {error && <div className="error-message">{error}</div>}
                {success && <div className="success-message">{success}</div>}
                <form onSubmit={handleSubmit}>
                    <div className="form-group">
                        <label htmlFor="name">Username</label>
                        <input
                            type="text"
                            id="name"
                            name="name"
                            value={formData.name}
                            onChange={handleChange}
                            required
                        />
                    </div>
                    <div className="form-group">
                        <label htmlFor="password">Password</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            value={formData.password}
                            onChange={handleChange}
                            required
                        />
                    </div>
                    <div className="form-group">
                        <label htmlFor="confirmPassword">Confirm Password</label>
                        <input
                            type="password"
                            id="confirmPassword"
                            name="confirmPassword"
                            value={formData.confirmPassword}
                            onChange={handleChange}
                            required
                        />
                    </div>
                    <button 
                        type="submit" 
                        className="auth-button"
                        disabled={isLoading}
                    >
                        {isLoading ? 'Registering...' : 'Register'}
                    </button>
                </form>
                <p className="auth-link">
                    Already have an account? <Link to="/login">Login here</Link>
                </p>
            </div>
        </div>
    );
};

export default Register;