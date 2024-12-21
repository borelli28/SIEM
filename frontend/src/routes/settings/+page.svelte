<script>
  let newAccount = { username: '', password: '', role: 'analyst' };
  let newLogSource = { name: '', type: 'syslog', address: '' };
  let newHost = { name: '', ip: '' };

  function createAccount(event) {
    event.preventDefault();
    console.log('Creating new account:', newAccount);
    // Implement account creation logic here
    newAccount = { username: '', password: '', role: 'analyst' };
  }

  function addLogSource(event) {
    event.preventDefault();
    console.log('Adding new log source:', newLogSource);
    // Implement log source addition logic here
    newLogSource = { name: '', type: 'syslog', address: '' };
  }

  function addHost(event) {
    event.preventDefault();
    console.log('Adding new host:', newHost);
    // Implement host addition logic here
    newHost = { name: '', ip: '' };
  }
</script>

<svelte:head>
  <link rel="stylesheet" href="/css/settings.css">
  <title>Settings</title>
</svelte:head>

<main>
  <div id="container">
    <h1>SIEM Settings</h1>

    <section>
      <h2>Create New Account</h2>
      <form on:submit={createAccount}>
        <div>
          <label for="username">Username:</label>
          <input type="text" id="username" bind:value={newAccount.username} required>
        </div>
        <div>
          <label for="password">Password:</label>
          <input type="password" id="password" bind:value={newAccount.password} required>
        </div>
        <div>
          <label for="role">Role:</label>
          <select id="role" bind:value={newAccount.role}>
            <option value="analyst">Analyst</option>
            <option value="admin">Admin</option>
          </select>
        </div>
        <button type="submit">Create Account</button>
      </form>
    </section>

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
    </nav>
  </div>
</main>