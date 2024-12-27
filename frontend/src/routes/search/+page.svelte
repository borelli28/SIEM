<script>
  import { onMount } from 'svelte';
  import { getCsrfToken } from '../../services/csrfService';
  import { isAuthenticated, checkAuth, logout } from '../../services/authService.js';

  let searchQuery = '';
  let startDate = '';
  let endDate = '';
  let logType = 'all';
  let severity = 'all';
  let logs = [];
  let alertMessage = '';
  let alertType = 'error';

  onMount(async () => {
    await checkAuth();
    if (!$isAuthenticated) {
      window.location.href = '/login';
      return;
    }
    // Initialize date inputs with today's date
    const today = new Date().toISOString().split('T')[0];
    startDate = today;
    endDate = today;
  });

  async function handleSearch(event) {
    event.preventDefault();
    console.log('Searching with criteria:', { searchQuery, startDate, endDate, logType, severity });
    // In a real application, this would be an API call
    // For now, we'll use mock data
    logs = [
      { id: 1, timestamp: '2023-12-08T10:30:00Z', type: 'system', message: 'System startup', severity: 'info' },
      { id: 2, timestamp: '2023-12-08T11:15:00Z', type: 'security', message: 'Failed login attempt', severity: 'warning' },
      { id: 3, timestamp: '2023-12-08T12:00:00Z', type: 'application', message: 'Database connection error', severity: 'error' },
      { id: 4, timestamp: '2023-12-08T13:45:00Z', type: 'network', message: 'Unusual outbound traffic detected', severity: 'critical' },
    ];
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
  <title>Log Search</title>
  <link rel="stylesheet" href="/css/search.css">
</svelte:head>

<main>
  <div class="container">
    <h1>SIEM Log Search</h1>

    <form on:submit={handleSearch}>
      <div class="search-controls">
        <input type="text" bind:value={searchQuery} placeholder="Search logs...">
        <input type="date" bind:value={startDate}>
        <input type="date" bind:value={endDate}>
        <select bind:value={logType}>
          <option value="all">All Log Types</option>
          <option value="system">System</option>
          <option value="security">Security</option>
          <option value="application">Application</option>
          <option value="network">Network</option>
        </select>
        <select bind:value={severity}>
          <option value="all">All Severities</option>
          <option value="info">Info</option>
          <option value="warning">Warning</option>
          <option value="error">Error</option>
          <option value="critical">Critical</option>
        </select>
        <button type="submit">Search</button>
      </div>
    </form>

    {#if logs.length > 0}
      <table>
        <thead>
          <tr>
            <th>Timestamp</th>
            <th>Type</th>
            <th>Message</th>
            <th>Severity</th>
          </tr>
        </thead>
        <tbody>
          {#each logs as log (log.id)}
            <tr class={log.severity}>
              <td>{new Date(log.timestamp).toLocaleString()}</td>
              <td>{log.type}</td>
              <td>{log.message}</td>
              <td>{log.severity}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p>No logs found. Try adjusting your search criteria.</p>
    {/if}

    <nav>
      <a href="/">Back to Dashboard</a>
      <button on:click={handleLogout} id="logout-btn">Logout</button>
    </nav>
  </div>
</main>