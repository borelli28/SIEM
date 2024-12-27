<script>
  let username = '';
  let password = '';
  let confirmPassword = '';
  let alertMessage = '';
  let alertType = '';

  async function handleSubmit() {
    if (password !== confirmPassword) {
      alertMessage = 'Passwords do not match!';
      alertType = 'error';
      return;
    }

    const newAccount = {
      id: "0",
      name: username,
      password: password,
      role: 'Admin'
    };

    try {
      const response = await fetch('http://localhost:4200/backend/account/', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(newAccount),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to create account.');
      }

      const data = await response.json();
      alertMessage = 'Account created successfully!';
      alertType = 'success';
      username = '';
      password = '';
      confirmPassword = '';
    } catch (error) {
      console.error("Error creating account:", error);
      alertMessage = `Error: ${error.message}`;
      alertType = 'error';
    }
  }
</script>

<svelte:head>
  <link rel="stylesheet" href="/css/login.css">
  <title>Register</title>
</svelte:head>

<main>
  <div id="container">
    <h1>Register</h1>
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
      <div>
        <label for="confirmPassword">Confirm Password:</label>
        <input type="password" id="confirmPassword" bind:value={confirmPassword} required>
      </div>
      <button type="submit">Register</button>
    </form>

    <p>Have an account? <a href="/login">Login here</a></p>
    <p>Visit <a href="https://svelte.dev/docs/kit" target="_blank">svelte.dev/docs/kit</a> to read the documentation</p>
  </div>
</main>