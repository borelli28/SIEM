import React, { useState, useEffect } from 'react';
import { getCsrfToken } from '../../services/csrfService';
import '../../styles/CaseTabs.css';

const ObservablesTab = ({ caseId, formId, showAlert }) => {
    const [observables, setObservables] = useState([]);
    const [showObservableForm, setShowObservableForm] = useState(false);

    useEffect(() => {
        fetchObservables();
    }, []);

    const fetchObservables = async () => {
        try {
            const response = await fetch(`http://localhost:4200/backend/case/${caseId}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch observables');
            }

            const data = await response.json();
            if (data && data.observables && Array.isArray(data.observables)) {
                setObservables(data.observables);
            } else {
                showAlert('Server did not return valid observables data');
                setObservables([]);
            }
        } catch (err) {
            showAlert('Failed to fetch observables');
        }
    };

    const handleAddObservable = async (e) => {
        e.preventDefault();
        const formData = {
            observable_type: e.target.type.value,
            value: e.target.value.value
        };

        try {
            const csrfToken = await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/case/${caseId}/observable`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include',
                body: JSON.stringify(formData)
            });

            if (!response.ok) {
                throw new Error('Failed to add observable');
            }

            showAlert('Observable added successfully', 'success');
            e.target.reset();
            setShowObservableForm(false);
            await fetchObservables();
        } catch (err) {
            showAlert('Failed to add observable');
        }
    };

    const handleDeleteObservable = async (observable) => {
        if (!window.confirm('Are you sure you want to delete this observable?')) {
            return;
        }

        try {
            const csrfToken = await getCsrfToken(formId);
            const response = await fetch(`http://localhost:4200/backend/case/${caseId}/observable`, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include',
                body: JSON.stringify({
                    observable_type: observable.observable_type,
                    value: observable.value
                })
            });

            if (!response.ok) {
                throw new Error('Failed to delete observable');
            }

            showAlert('Observable deleted successfully', 'success');
            await fetchObservables();
        } catch (err) {
            showAlert('Failed to delete observable');
        }
    };

    return (
        <div className="tab-container">
            <div className="observables-section">
                <div className="add-observable-container">
                    <form onSubmit={handleAddObservable}>
                        <select name="type" required>
                            <option value="ip">IP Address</option>
                            <option value="domain">Domain</option>
                            <option value="hash">File Hash</option>
                            <option value="url">URL</option>
                        </select>
                        <input 
                            type="text" 
                            name="value" 
                            placeholder="Observable value"
                            required 
                        />
                        <button type="submit" className="primary-btn">Add Observable</button>
                    </form>
                </div>

                <div className="observables-list">
                    {observables.map(observable => (
                        <div key={observable.id} className="observable">
                            <div className="observable-header">
                                <div className="observable-content">
                                    <strong>{observable.observable_type}:</strong> {observable.value}
                                </div>
                                <button
                                    className="delete-observable-btn"
                                    onClick={() => handleDeleteObservable(observable)}
                                >
                                    Delete
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export default ObservablesTab;