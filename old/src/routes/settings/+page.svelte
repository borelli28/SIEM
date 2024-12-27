<script>
  import { onMount } from 'svelte';
  import { getCsrfToken } from '../../services/csrfService';
  import { isAuthenticated, checkAuth, logout, user } from '../../services/authService.js';
  import { get } from 'svelte/store'; // Import get to read store values

  let newLogSource = { name: '', type: 'syslog', address: '' };
  let newHost = { hostname: '', ip: '' };
  let alertMessage = '';
  let alertType = 'error';
  let formId = 'setting-form';
  let accountId = '';

  onMount(async () => {
    await checkAuth();

    // Get the authentication status and user information
    const authStatus = get(isAuthenticated); // Get current authentication status
    const currentUser = get(user); // Get the current user information

    // Check if the user is authenticated
    if (!authStatus) {
      window.location.href = '/login';
      return;
    }

    // Set accountId based on the user information from the store
    if (currentUser) {
      accountId = currentUser.accountId; // Adjust this depending on the actual structure of your user object
      console.log(`Account ID: ${accountId}`); // For debugging purposes
    }

    try {
      await getCsrfToken(formId);
    } catch (error) {
      alertMessage = 'Failed to fetch CSRF token';
      alertType = 'error';
    }
  });

  function addLogSource(event) {
    event.preventDefault();
    console.log('Adding new log source:', newLogSource);
    newLogSource = { name: '', type: 'syslog', address: '' };
  }

  async function addHost(event) {
    event.preventDefault();
    newHost = { hostname: '', ip: '' };

    try {
      const response = await fetch(`http://localhost:4200/backend/host/${accountId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Form-ID': formId
        },
        body: JSON.stringify(newHost),
        credentials: 'include'
      });

      if (response.ok) {
        const data = await response.json(); // Await the response.json() correctly
        alertType = 'success';
        alertMessage = 'New host added';
      } else {
        alertType = 'error';
        alertMessage = 'Could not add new host';
      }
    } catch (error) {
      console.error('Error adding host:', error);
      alertType = 'error';
      alertMessage = 'An error occurred during the request';
    }
  }

  async function handleLogout() {
    const result = await logout();
    if (!result.success) {
      console.log(result.message);
      alertType = 'error';
      alertMessage = 'Logout unsuccessful';
    } else {
      alertType = 'success';
      alertMessage = 'Logout successful';
      window.location.href = '/login';
    }
  }
</script>

<svelte:head>
  <link rel="stylesheet" href="/css/settings.css">
  <title>Settings</title>
</svelte:head>

<main>
  <div id="container">
    <h1>SIEM Settings</h1>

    {#if alertMessage}
      <div class={`alert ${alertType}`}>
        {alertMessage}
      </div>
    {/if}

    <section>
      <h2>Add New Log Source</h2>
      <form on:submit={addLogSource}>
        <div>
          <label for="sourceName">Source Name:</label>
          <input type="text" id="sourceName" bind:value={newLogSource.name} required>
        </div>
        <div>
          <label for="sourceType">Source Type:</label>
          <select id="sourceType" bind:value={newLogSource.type}>
            <option value="syslog">Syslog</option>
            <option value="winlog">Windows Event Log</option>
            <option value="apache">Apache Log</option>
          </select>
        </div>
        <div>
          <label for="sourceAddress">Source Address:</label>
          <input type="text" id="sourceAddress" bind:value={newLogSource.address} required>
        </div>
        <button type="submit">Add Log Source</button>
      </form>
    </section>

    <section>
      <h2>Add New Host</h2>
      <form on:submit={addHost}>
        <div>
          <label for="hostName">Host Name:</label>
          <input type="text" id="hostName" bind:value={newHost.hostname} required>
        </div>
        <div>
          <label for="hostIP">IP Address:</label>
          <input type="text" id="hostIP" bind:value={newHost.ip} required>
        </div>
        <button type="submit">Add Host</button>
      </form>
    </section>

    <nav>
      <a href="/">Back to Dashboard</a>
      <button on:click={handleLogout} id="logout-btn">Logout</button>
    </nav>
  </div>
</main>