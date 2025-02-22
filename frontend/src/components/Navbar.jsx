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
        <div className="container">
            <nav>
                <a href="/dashboard">Dashboard</a>
                <a href="/settings">Settings</a>
                <a href="/alerts">Alerts</a>
                <a href="/search">Search</a>
                <a href="/list-cases">Cases</a>
                <a className="danger-btn" onClick={handleLogout}>Logout</a>
            </nav>
        </div>
    );
};

export default Navbar;