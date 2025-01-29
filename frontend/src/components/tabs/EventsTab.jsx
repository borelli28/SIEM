import React, { useState, useEffect } from 'react';
import '../../styles/CaseTabs.css';

const EventsTab = ({ caseId, formId, showAlert }) => {
    const [events, setEvents] = useState([]);

    useEffect(() => {
        fetchEvents();
    }, []);

    //
    // Events are observables of type "alert" & "log"
    //

    const fetchEvents = async () => {
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
                throw new Error('Failed to fetch events');
            }

            const data = await response.json();

            if (data && Array.isArray(data.observables)) {
                const filteredEvents = data.observables.filter(observable =>
                    observable.observable_type === 'alert' || observable.observable_type === 'log'
                );
                setEvents(filteredEvents);
            } else {
                showAlert('No events found in this case.');
            }
        } catch (err) {
            showAlert('Failed to fetch events');
        }
    };

    return (
        <div className="tab-container">
            <div className="events-section">
                {events.map(event => (
                    <div key={event.id} className={`event ${event.observable_type}-event`}>
                        <h4>{event.observable_type === 'alert' ? 'Alert Event' : 'Log Event'}</h4>
                        <div id="event-value">
                            <p>{JSON.stringify(event.value, null, 2)}</p>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default EventsTab;