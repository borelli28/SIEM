async function handleSubmit(event) {
    event.preventDefault();
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    const confirmPassword = document.getElementById('confirmPassword').value;

    const alertContainer = document.getElementById('alert-container');

    if (password !== confirmPassword) {
        alertContainer.innerHTML = `<div class="alert error">Passwords do not match!</div>`;
        return;
    }

    const newAccount = { name: username, password: password };

    try {
        const response = await fetch('http://localhost:4200/backend/account/', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(newAccount),
        });

        if (response.ok) {
            alertContainer.innerHTML = `<div class="alert success">Account created successfully!</div>`;
            document.getElementById('username').value = '';
            document.getElementById('password').value = '';
            document.getElementById('confirmPassword').value = '';
        } else {
            const errorData = await response.json();
            alertContainer.innerHTML = `<div class="alert error">Error: ${errorData.message || 'Failed to create account.'}</div>`;
        }
    } catch (error) {
        alertContainer.innerHTML = `<div class="alert error">Error creating account: ${error.message}</div>`;
    }
}