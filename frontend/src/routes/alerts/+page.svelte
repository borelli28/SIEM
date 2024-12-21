<script>
  import { onMount } from 'svelte';

  let alerts = [];
  let sortField = 'timestamp';
  let sortDirection = 'desc';
  let filterSeverity = 'all';

  onMount(async () => {
    // Fetch alerts data
    await fetchAlerts();
  });

  async function fetchAlerts() {
    // In a real application, this would be an API call
    // For now, we'll use mock data
    alerts = [
      { id: 1, ruleName: 'Suspicious Login', host: '192.168.1.100', severity: 'High', timestamp: '2023-12-08T10:30:00Z' },
      { id: 2, ruleName: 'Failed SSH Attempts', host: '192.168.1.101', severity: 'Medium', timestamp: '2023-12-08T11:15:00Z' },
      { id: 3, ruleName: 'Unusual Network Traffic', host: '192.168.1.102', severity: 'Low', timestamp: '2023-12-08T12:00:00Z' },
      { id: 4, ruleName: 'Malware Detected', host: '192.168.1.103', severity: 'Critical', timestamp: '2023-12-08T13:45:00Z' },
    ];
  }

  function sortAlerts(field) {
    if (sortField === field) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortField = field;
      sortDirection = 'asc';
    }

    alerts = alerts.sort((a, b) => {
      if (a[field] < b[field]) return sortDirection === 'asc' ? -1 : 1;
      if (a[field] > b[field]) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });
  }

  function filterAlerts() {
    if (filterSeverity === 'all') {
      fetchAlerts();
    } else {
      alerts = alerts.filter(alert => alert.severity.toLowerCase() === filterSeverity.toLowerCase());
    }
  }
</script>

<svelte:head>
  <title>Alerts</title>
  <link rel="stylesheet" href="/css/alerts.css">
</svelte:head>

<main>
  <div class="container">
    <h1>SIEM Alerts</h1>

    <div class="filter-container">
      <label for="severity-filter">Filter by Severity:</label>
      <select id="severity-filter" bind:value={filterSeverity} on:change={filterAlerts}>
        <option value="all">All</option>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </div>

    <table>
      <thead>
        <tr>
          <th on:click={() => sortAlerts('ruleName')}>Rule Name</th>
          <th on:click={() => sortAlerts('host')}>Host</th>
          <th on:click={() => sortAlerts('severity')}>Severity</th>
          <th on:click={() => sortAlerts('timestamp')}>Timestamp</th>
        </tr>
      </thead>
      <tbody>
        {#each alerts as alert (alert.id)}
          <tr class={alert.severity.toLowerCase()}>
            <td>{alert.ruleName}</td>
            <td>{alert.host}</td>
            <td>{alert.severity}</td>
            <td>{new Date(alert.timestamp).toLocaleString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    <nav>
      <a href="/">Back to Dashboard</a>
    </nav>
  </div>
</main>