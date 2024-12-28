let isAuthenticated = false;
let user = null;

export async function checkAuth() {
    try {
        const response = await fetch('http://localhost:4200/backend/check-auth', {
            credentials: 'include'
        });
        if (response.ok) {
            const data = await response.json();
            isAuthenticated = true;
            user = data.user;
        } else {
            isAuthenticated = false;
            user = null;
        }
    } catch (error) {
        console.error('Error checking auth:', error);
        isAuthenticated = false;
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
            isAuthenticated = false;
            user = null;
            return { success: true };
        }
        return { success: false, message: "Logout didn't work" };
    } catch (error) {
        return { success: false, message: "Error logging out" };
    }
}

export { isAuthenticated, user };