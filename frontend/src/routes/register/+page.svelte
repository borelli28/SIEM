<script>
  let username = '';
  let password = '';
  let confirmPassword = '';

  function handleSubmit() {
    // Here you would typically handle the registration logic
    console.log('Registration attempted', { username, password, confirmPassword });
    // You can add your authentication logic here
  }
</script>

<svelte:head>
	  <link rel="stylesheet" href="/css/login.css">
	  <title>Register</title>
</svelte:head>

<main>
		<div id="container">
				<h1>Register</h1>
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