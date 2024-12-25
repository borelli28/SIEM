<script>
  import { isAuthenticated, user } from '../../stores/authStore.js';

  let username = '';
  let password = '';
  let alertMessage = '';
  let alertType = 'error';

  async function handleLogin(event) {
    event.preventDefault();
    alertMessage = '';

    const loginData = {
      id: "0",
      name: username,
      password: password,
      role: "no"
    };

    try {
      const response = await fetch('http://localhost:4200/backend/account/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(loginData),
        credentials: 'include'
      });

      if (response.ok) {
        const data = await response.json();
        isAuthenticated.set(true);
        user.set(data.user);
        alertType = 'success';
        alertMessage = 'Login successful! Redirecting...';
        setTimeout(() => {
          window.location.href = '/';
        }, 1500); // Redirect after 1.5 seconds
      } else {
        alertType = 'error';
        alertMessage = 'Invalid credentials';
      }
    } catch (error) {
      console.error('Login error:', error);
      alertType = 'error';
      alertMessage = 'An error occurred during login';
    }
  }
</script>

<svelte:head>
  <link rel="stylesheet" href="/css/login.css">
  <title>Login</title>
</svelte:head>

<main>
  <div id="container">
    <h1>Login</h1>
    {#if alertMessage}
      <div class={`alert ${alertType}`}>
        {alertMessage}
      </div>
    {/if}
    <form on:submit={handleLogin}>
      <div>
        <label for="username">Username:</label>
        <input type="text" id="username" bind:value={username} required>
      </div>
      <div>
        <label for="password">Password:</label>
        <input type="password" id="password" bind:value={password} required>
      </div>
      <button type="submit">Log In</button>
    </form>
    <p>New to our platform? <a href="/register">Register here</a></p>
    <p>Visit <a href="https://svelte.dev/docs" target="_blank">svelte.dev/docs</a> to read the documentation</p>
  </div>
</main>