<script>
  let username = '';
  let password = '';
  let alertMessage = '';
  let alertType = '';

  async function handleSubmit() {
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
      });

      const contentType = response.headers.get('Content-Type') || '';
      let errorMessage = '';

      if (!response.ok) {
        if (contentType.includes('application/json')) {
          const errorData = await response.json();
          errorMessage = errorData.message || 'Failed to log in.';
        } else {
          const errorText = await response.text();
          errorMessage = `Unexpected response: ${errorText}`;
        }
        throw new Error(errorMessage);
      }

      if (contentType.includes('application/json')) {
        const data = await response.json();
        alertMessage = 'Login successful!';
        alertType = 'success';
      } else {
        alertMessage = 'Login successful, but response is not JSON.';
        alertType = 'success';
      }
    } catch (error) {
      console.error('Error logging in:', error);
      alertMessage = `Error: ${error.message}`;
      alertType = 'error';
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
    <form on:submit|preventDefault={handleSubmit}>
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
    <p>Visit <a href="https://svelte.dev/docs/kit" target="_blank">svelte.dev/docs/kit</a> to read the documentation</p>
  </div>
</main>