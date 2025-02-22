import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import '../styles/Navbar.css';

const Navbar = () => {
    const navigate = useNavigate();

    const handleLogout = async () => {
        try {
            const response = await fetch('http://localhost:4200/backend/logout', {
                method: 'POST',
                credentials: 'include'
            });

            if (response.ok) {
                navigate('/login');
            } else {
                console.error('Logout failed');
            }
        } catch (err) {
            console.error('Error during logout:', err);
        }
    };

    return (
        <div className="nav-container">
            <nav>
                <a href="/dashboard" className="primary-btn">Dashboard</a>
                <a href="/settings" className="primary-btn">Settings</a>
                <a href="/alerts" className="primary-btn">Alerts</a>
                <a href="/search" className="primary-btn">Search</a>
                <a href="/list-cases" className="primary-btn">Cases</a>
                <a className="danger-btn" onClick={handleLogout}>Logout</a>
            </nav>
        </div>
    );
};

export default Navbar;