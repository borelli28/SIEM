import React, { useState, useEffect } from 'react';

const EventsTab = ({ caseId, formId, showAlert }) => {
    const [events, setEvents] = useState([]);

    useEffect(() => {
        fetchEvents();
    }, []);

    const fetchEvents = async () => {
        try {
            const response = await fetch(`http://localhost:4200/backend/case/${caseId}/events`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Form-ID': formId
                },
                credentials: 'include'
            });

            if (!response.ok) {
                throw new Error('Failed to fetch events');
            }

            const data = await response.json();
            setEvents(data);
        } catch (err) {
            showAlert('Failed to fetch events');
        }
    };

    return (
        <div className="events-section">
            {events.map(event => (
                <div key={event.id} className={`event ${event.type}-event`}>
                    <h4>{event.type === 'alert' ? 'Alert Event' : 'Log Event'}</h4>
                    <pre>{JSON.stringify(JSON.parse(event.data), null, 2)}</pre>
                    <p>Added on {new Date(event.created_at).toLocaleString()}</p>
                </div>
            ))}
        </div>
    );
};

export default EventsTab;