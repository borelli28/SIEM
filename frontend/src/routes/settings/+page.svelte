<script>
  import { onMount } from 'svelte';
  import { getCsrfToken } from '../../services/csrfService';
  import { isAuthenticated, checkAuth, logout } from '../../services/authService.js';

  let newLogSource = { name: '', type: 'syslog', address: '' };
  let newHost = { name: '', ip: '' };
  let alertMessage = '';
  let alertType = 'error';
  let formId = 'setting-form';

  onMount(async () => {
    await checkAuth();
    if (!$isAuthenticated) {
      window.location.href = '/login';
      return;
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

  function addHost(event) {
    event.preventDefault();
    console.log('Adding new host:', newHost);
    newHost = { name: '', ip: '' };
  }

  async function handleLogout() {
      const result = await logout();
      if (!result.success) {
          console.log(result.message);
          alertType = 'error';
          alertMessage = 'Logout unsucesful';
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
          <input type="text" id="hostName" bind:value={newHost.name} required>
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