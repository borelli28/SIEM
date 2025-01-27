let isAuthenticated = false;
let user = null;

export function getAuthenticationStatus() {
    return isAuthenticated;
}

export function setAuthenticationStatus(status) {
    isAuthenticated = status;
}

export async function checkAuth() {
    try {
        const response = await fetch('http://localhost:4200/backend/check-auth', {
            credentials: 'include'
        });
        if (response.ok) {
            const data = await response.json();
            setAuthenticationStatus(true);
            user = data.account_id;
        } else {
            setAuthenticationStatus(false);
            user = null;
        }
    } catch (error) {
        console.error('Error checking auth:', error);
        setAuthenticationStatus(false);
        user = null;
    }
}

export async function logout() {
    try {
        const response = await fetch('http://localhost:4200/backend/logout', {
            method: 'POST',
            credentials: 'include'
        });
        if (response.ok) {
            setAuthenticationStatus(false);
            user = null;
            return { success: true };
        }
        return { success: false, message: "Logout didn't work" };
    } catch (error) {
        return { success: false, message: "Error logging out" };
    }
}

export { user };