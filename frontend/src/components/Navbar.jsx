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
                <Link to="/dashboard">Dashboard</Link>
                <Link to="/settings">Settings</Link>
                <Link to="/alerts">Alerts</Link>
                <Link to="/search">Search</Link>
                <Link to="/list-cases">Cases</Link>
                <Link id="logout-btn" onClick={handleLogout}>Logout</Link>
            </nav>
        </div>
    );
};

export default Navbar;