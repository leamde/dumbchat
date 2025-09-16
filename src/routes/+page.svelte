<script>
  import { invoke } from "@tauri-apps/api/core";

  let username = "";
  let password = "";
  let recoveryKey = "";
  let createdPassword = "";
  let message = "";
  let showRecoveryKey = false;

  async function login() {
    console.log("Login button clicked", { username, password, recoveryKey });
    if (!username || (!password && !recoveryKey)) {
      message = "Please enter username and a password or recovery key";
      console.error("Login validation failed: missing username or credentials");
      return;
    }
    try {
      console.log("Invoking login command...");
      const response = await invoke("login", {
        username,
        password,
        recoveryKey: recoveryKey || null,
      });
      console.log("Login response:", JSON.stringify(response, null, 2));
      message = response.message;
      if (response.success) {
        console.log("Navigating to /inbox with full reload");
        try {
          window.location.href = "/inbox";
          console.log("Navigation to /inbox triggered successfully");
        } catch (navError) {
          message = `Navigation to inbox failed: ${navError.message || navError}`;
          console.error("Navigation error:", JSON.stringify(navError, null, 2));
        }
      } else {
        console.error("Login failed:", response.message);
      }
    } catch (error) {
      message = `Login failed: ${error.message || error}`;
      console.error("Login error:", JSON.stringify(error, null, 2));
    }
  }

  async function createInstance() {
    console.log("Create instance button clicked", { username, password });
    if (!username || !password) {
      message = "Please enter a username and password";
      console.error(
        "Create instance validation failed: missing username or password",
      );
      return;
    }
    try {
      console.log("Invoking create_account command...");
      const response = await invoke("create_account", { username, password });
      console.log(
        "Create account response:",
        JSON.stringify(response, null, 2),
      );
      message = response.message;
      if (response.success && response.data) {
        recoveryKey = response.data;
        showRecoveryKey = true;
        createdPassword = password;
        password = "";
        console.log("Account created, recovery key:", recoveryKey);
      } else {
        console.error("Create account failed:", response.message);
      }
    } catch (error) {
      message = `Error creating account: ${error.message || error}`;
      console.error("Create account error:", JSON.stringify(error, null, 2));
    }
  }

  async function continueToChat() {
    console.log("Continue to chat button clicked", {
      username,
      createdPassword,
    });
    if (!username || !createdPassword) {
      message = "Please re-enter username and try again";
      console.error(
        "Continue to chat validation failed: missing username or password",
      );
      return;
    }
    try {
      console.log("Invoking login command for continue...");
      const response = await invoke("login", {
        username,
        password: createdPassword,
        recoveryKey: null,
      });
      console.log(
        "Continue login response:",
        JSON.stringify(response, null, 2),
      );
      message = response.message;
      if (response.success) {
        console.log("Navigating to /inbox with full reload");
        try {
          window.location.href = "/inbox";
          console.log("Navigation to /inbox triggered successfully");
        } catch (navError) {
          message = `Navigation to inbox failed: ${navError.message || navError}`;
          console.error("Navigation error:", JSON.stringify(navError, null, 2));
        }
      } else {
        console.error("Continue login failed:", response.message);
      }
    } catch (error) {
      message = `Login failed: ${error.message || error}`;
      console.error("Continue login error:", JSON.stringify(error, null, 2));
    }
  }

  async function copyRecoveryKey() {
    try {
      console.log("Copying recovery key to clipboard...");
      await navigator.clipboard.writeText(recoveryKey);
      message = "Recovery key copied to clipboard!";
      console.log("Recovery key copied successfully");
    } catch (error) {
      message = `Failed to copy recovery key: ${error.message || error}`;
      console.error("Copy recovery key error:", JSON.stringify(error, null, 2));
    }
  }
</script>

<main>
  <h1>DumbChat</h1>
  <div class="options">
    <input type="text" bind:value={username} placeholder="Enter username" />
    <input type="password" bind:value={password} placeholder="Enter password" />
    <input
      type="text"
      bind:value={recoveryKey}
      placeholder="Enter recovery key (optional for login)"
    />
    <button on:click={login}>Login</button>
    <button on:click={createInstance}>Create New Instance</button>
    <p class="info">
      To create an account, enter a username and password, then click "Create
      New Instance" to generate a recovery key. Save the key, then click "I've
      saved it, continue" or use "Login" to access your inbox. After logging in,
      you can compose rich text messages.
    </p>
  </div>
  {#if message}
    <p>{message}</p>
  {/if}
  {#if showRecoveryKey}
    <p><strong>Recovery Key: {recoveryKey}</strong></p>
    <p>
      Please securely store your recovery key. It will only be displayed once
      and is required to recover your account if you forget your password.
    </p>
    <button on:click={copyRecoveryKey}>Copy to Clipboard</button>
    <button on:click={continueToChat}>I've saved it, continue</button>
  {/if}
</main>

<style>
  main {
    text-align: center;
    padding: 2rem;
    max-width: 400px;
    margin: 0 auto;
  }
  h1 {
    color: #333;
    font-size: 2.5rem;
    margin-bottom: 1.5rem;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  input {
    padding: 0.8rem;
    font-size: 1.2rem;
    border: 1px solid #ccc;
    border-radius: 5px;
  }
  button {
    padding: 0.8rem 1.5rem;
    font-size: 1.2rem;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    transition: background-color 0.2s;
    margin-top: 0.5rem;
  }
  button:hover {
    background-color: #0056b3;
  }
  p {
    margin-top: 1rem;
    color: #333;
  }
  p.info {
    font-size: 0.9rem;
    color: #666;
  }
  p strong {
    color: #d32f2f;
  }
</style>
