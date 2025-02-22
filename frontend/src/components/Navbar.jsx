import React, { useState, useEffect, useRef } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import '../styles/Navbar.css';

const Navbar = () => {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const navigate = useNavigate();
    const menuRef = useRef(null); // Track the mobile menu content

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

    const toggleMenu = () => {
        setIsMenuOpen(!isMenuOpen);
    };

    // Close the menu when clicking outside
    useEffect(() => {
        const handleClickOutside = (event) => {
            if (isMenuOpen && menuRef.current && !menuRef.current.contains(event.target)) {
                setIsMenuOpen(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            // Unbind the event listener on cleanup
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isMenuOpen]); // Re-run effect when isMenuOpen changes

    return (
        <div className="nav-container">
            <nav>
                <div className="nav-links desktop">
                    <a href="/dashboard" className="primary-btn">Dashboard</a>
                    <a href="/settings" className="primary-btn">Settings</a>
                    <a href="/alerts" className="primary-btn">Alerts</a>
                    <a href="/search" className="primary-btn">Search</a>
                    <a href="/list-cases" className="primary-btn">Cases</a>
                    <a className="danger-btn" onClick={handleLogout}>Logout</a>
                </div>

                <div className="hamburger" onClick={toggleMenu}>
                    <span></span>
                    <span></span>
                    <span></span>
                </div>

                {/* Mobile Full-Screen Menu (Hidden by default, shown when isMenuOpen is true) */}
                {isMenuOpen && (
                    <div className="mobile-menu" onClick={toggleMenu}> {/* Click on overlay closes menu */}
                        <div className="menu-content" ref={menuRef} onClick={e => e.stopPropagation()}> {/* Prevent click propagation on buttons */}
                            <a href="/dashboard" className="primary-btn" onClick={toggleMenu}>Dashboard</a>
                            <a href="/settings" className="primary-btn" onClick={toggleMenu}>Settings</a>
                            <a href="/alerts" className="primary-btn" onClick={toggleMenu}>Alerts</a>
                            <a href="/search" className="primary-btn" onClick={toggleMenu}>Search</a>
                            <a href="/list-cases" className="primary-btn" onClick={toggleMenu}>Cases</a>
                            <a className="danger-btn" onClick={() => { handleLogout(); toggleMenu(); }}>Logout</a>
                        </div>
                    </div>
                )}
            </nav>
        </div>
    );
};

export default Navbar;