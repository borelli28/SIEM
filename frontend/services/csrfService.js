export async function getCsrfToken(formId) {
    try {
        const response = await fetch('http://localhost:4200/backend/csrf/', {
            method: 'GET',
            headers: { 'X-Form-ID': formId },
            credentials: 'include'
        });
        if (!response.ok) {
            throw new Error('Failed to get CSRF token');
        }
        const data = await response.json();
        return data.csrfToken;
    } catch (error) {
        console.error('Error getting CSRF token:', error);
        throw error;
    }
}