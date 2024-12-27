export async function getCsrfToken(formId) {
    try {
        const response = await fetch('http://localhost:4200/backend/csrf/', {
            method: 'GET',
            headers: {
                'X-Form-ID': formId
            },
            credentials: 'include'
        });

        if (response.ok) {
            return response.ok
        } else {
            console.error('Failed to get CSRF token');
            throw new Error('Failed to get CSRF token');
        }
    } catch (error) {
        console.error('Error getting CSRF token:', error);
        throw error;
    }
}