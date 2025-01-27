import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import '../styles/Navbar.css';

const Navbar = () => {
    const navigate = useNavigate();

    const handleLogout = async () => {
        try {
            const response = await fetch('http://localhost:4200/backend/account/logout', {
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
        <nav>
            <Link to="/">Dashboard</Link>
            <Link to="/settings">Settings</Link>
            <Link to="/alerts">All Alerts</Link>
            <Link to="/search">Search</Link>
            <Link to="/list-cases">Cases</Link>
            <button id="logout-btn" onClick={handleLogout}>Logout</button>
        </nav>
    );
};

export default Navbar;