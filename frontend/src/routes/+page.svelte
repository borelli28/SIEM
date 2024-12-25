<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';
  import { isAuthenticated, checkAuth, logout } from '../stores/authStore.js';

  let alertMessage = '';
  let alertType = 'error';

  let searchQuery = '';
  let alerts = [
    { id: 1, ruleName: 'Suspicious Login', host: '192.168.1.100', severity: 'High' },
    { id: 2, ruleName: 'Failed SSH Attempts', host: '192.168.1.101', severity: 'Medium' },
    { id: 3, ruleName: 'Unusual Network Traffic', host: '192.168.1.102', severity: 'Low' },
  ];

  onMount(async () => {
    await checkAuth();
    if (!$isAuthenticated) {
      window.location.href = '/login';
      return;
    }

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
  <title>Dashboard</title>
  <link rel="stylesheet" href="/css/dashboard.css">
</svelte:head>

{#if $isAuthenticated}
  <main>
    <div id="container">
      <h1>SIEM Dashboard</h1>

      {#if alertMessage}
        <div class={`alert ${alertType}`}>
          {alertMessage}
        </div>
      {/if}

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

      <nav>
        <a href="/settings">Settings</a>
        <a href="/alerts">All Alerts</a>
        <a href="/search">Search</a>
        <button on:click={handleLogout} id="logout-btn">Logout</button>
      </nav>

      <p>Visit <a href="https://svelte.dev/docs" target="_blank">svelte.dev/docs</a> to read the documentation</p>
    </div>
  </main>
{:else}
  <p>Checking authentication...</p>
{/if}