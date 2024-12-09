<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';

  let searchQuery = '';
  let alerts = [
    { id: 1, ruleName: 'Suspicious Login', host: '192.168.1.100', severity: 'High' },
    { id: 2, ruleName: 'Failed SSH Attempts', host: '192.168.1.101', severity: 'Medium' },
    { id: 3, ruleName: 'Unusual Network Traffic', host: '192.168.1.102', severity: 'Low' },
  ];

  onMount(() => {
    // Example chart
    const ctx = document.getElementById('logsChart');
    new Chart(ctx, {
      type: 'bar',
      data: {
        labels: ['SSH', 'HTTP', 'FTP', 'DNS', 'SMTP'],
        datasets: [{
          label: '# of Logs',
          data: [12, 19, 3, 5, 2],
          backgroundColor: [
            'rgba(255, 99, 132, 0.2)',
            'rgba(54, 162, 235, 0.2)',
            'rgba(255, 206, 86, 0.2)',
            'rgba(75, 192, 192, 0.2)',
            'rgba(153, 102, 255, 0.2)'
          ],
          borderColor: [
            'rgba(255, 99, 132, 1)',
            'rgba(54, 162, 235, 1)',
            'rgba(255, 206, 86, 1)',
            'rgba(75, 192, 192, 1)',
            'rgba(153, 102, 255, 1)'
          ],
          borderWidth: 1
        }]
      },
      options: {
        scales: {
          y: {
            beginAtZero: true
          }
        }
      }
    });
  });

  function handleSearch() {
    console.log('Searching for:', searchQuery);
    // Implement your search logic here
  }
</script>

<svelte:head>
  <title>SIEM Dashboard</title>
  <link rel="stylesheet" href="/css/dashboard.css">
</svelte:head>

<main>
  <div id="container">
    <h1>SIEM Dashboard</h1>

    <section id="graphs">
      <h2>Log Analysis</h2>
      <canvas id="logsChart"></canvas>
    </section>

    <section id="alerts">
      <h2>Recent Alerts</h2>
      <table>
        <thead>
          <tr>
            <th>Rule Name</th>
            <th>Host</th>
            <th>Severity</th>
          </tr>
        </thead>
        <tbody>
          {#each alerts as alert}
            <tr>
              <td>{alert.ruleName}</td>
              <td>{alert.host}</td>
              <td>{alert.severity}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section id="search">
      <h2>Log Search</h2>
      <form on:submit|preventDefault={handleSearch}>
        <input type="text" bind:value={searchQuery} placeholder="Enter search criteria...">
        <button type="submit">Search</button>
      </form>
    </section>

    <nav>
      <a href="/settings">Settings</a>
      <a href="/alerts">All Alerts</a>
    </nav>

    <p>Visit <a href="https://svelte.dev/docs/kit" target="_blank">svelte.dev/docs/kit</a> to read the documentation</p>
  </div>
</main>