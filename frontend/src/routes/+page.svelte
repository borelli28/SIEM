<svelte:head>
  <link rel="stylesheet" href="/css/login.css">
  <title>Login</title>
</svelte:head>

<script>
  let username = '';
  let password = '';

  function handleSubmit() {
    // Here you would typically handle the login logic
    console.log('Login attempted', { username, password });
    // You can add your authentication logic here
  }
</script>

<main>
	<div id="container">
		<h1>Login</h1>
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