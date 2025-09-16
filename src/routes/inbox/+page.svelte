<script>
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import sanitizeHtml from "sanitize-html";

    let messages = [];
    let error = null;
    let loading = true;
    let tauriCore = null;
    let tauriEvent = null;
    let username = "";
    let nostrPublic = "";
    let x25519Public = "";

    onMount(async () => {
        console.log("Inbox page mounted at", new Date().toISOString());
        await new Promise((resolve) => setTimeout(resolve, 100));
        if (!window.__TAURI__) {
            error = "Tauri runtime not available";
            console.error("Tauri runtime not available");
            loading = false;
            return;
        }

        loading = true;
        try {
            console.log("Dynamically importing @tauri-apps/api/core...");
            tauriCore = await import("@tauri-apps/api/core");
            console.log("Tauri core imported successfully");
            console.log("Dynamically importing @tauri-apps/api/event...");
            tauriEvent = await import("@tauri-apps/api/event");
            console.log("Tauri event imported successfully");

            console.log("Setting up new_message listener...");
            const unlisten = await tauriEvent.listen("new_message", (event) => {
                const payload = event.payload || {};
                console.log(
                    "Received new_message:",
                    JSON.stringify(payload, null, 2),
                );
                messages = [
                    ...messages,
                    {
                        sender_npub:
                            payload.sender_npub?.slice(0, 12) + "..." ||
                            "Unknown",
                        text: payload.text || "",
                        timestamp: payload.timestamp
                            ? new Date(
                                  payload.timestamp * 1000,
                              ).toLocaleString()
                            : "Unknown",
                    },
                ];
            });
            console.log("new_message listener set up successfully");

            console.log("Invoking get_user_info...");
            const userInfoResponse = await tauriCore.invoke("get_user_info");
            console.log(
                "get_user_info response:",
                JSON.stringify(userInfoResponse, null, 2),
            );
            if (!userInfoResponse.success) {
                error = userInfoResponse.message;
                console.error("User info error:", userInfoResponse.message);
                loading = false;
                return unlisten;
            }
            let data;
            try {
                data = JSON.parse(userInfoResponse.data || "{}");
                username = data.username || "Unknown";
                nostrPublic = data.nostr_public || "Not available";
                x25519Public = data.x25519_public || "Not available";
            } catch (parseError) {
                error = `Failed to parse user info: ${parseError}`;
                console.error(
                    "User info parse error:",
                    JSON.stringify(parseError, null, 2),
                );
                loading = false;
                return unlisten;
            }
            console.log("User info parsed:", JSON.stringify(data, null, 2));

            console.log("Invoking init_nostr_client...");
            const initResponse = await tauriCore.invoke("init_nostr_client");
            console.log(
                "init_nostr_client response:",
                JSON.stringify(initResponse, null, 2),
            );
            if (!initResponse.success) {
                error = initResponse.message;
                console.error("Nostr client error:", initResponse.message);
                loading = false;
                return unlisten;
            }

            console.log("Invoking receive_nostr_messages...");
            const receiveResponse = await tauriCore.invoke(
                "receive_nostr_messages",
            );
            console.log(
                "receive_nostr_messages response:",
                JSON.stringify(receiveResponse, null, 2),
            );
            if (!receiveResponse.success) {
                error = receiveResponse.message;
                console.error("Receive error:", receiveResponse.message);
                loading = false;
                return unlisten;
            }

            loading = false;
            console.log(
                "Inbox initialization complete at",
                new Date().toISOString(),
            );
            return unlisten;
        } catch (err) {
            error = `Inbox initialization failed: ${err.message || err}`;
            console.error("Inbox init error:", JSON.stringify(err, null, 2));
            loading = false;
        }
    });

    function goToCompose() {
        console.log("Navigating to compose page");
        goto("/compose");
    }
</script>

<main>
    <h1>Inbox</h1>
    {#if loading}
        <p>Loading...</p>
    {/if}
    {#if error}
        <p style="color: red;">{error}</p>
    {/if}
    {#if !loading && !error}
        <div>
            <h2>User Info</h2>
            <p>Username: {username}</p>
            <p>Nostr Public Key: {nostrPublic}</p>
            <p>X25519 Public Key: {x25519Public}</p>
        </div>
        <div>
            <h2>Messages</h2>
            {#each messages as msg}
                <div class="message">
                    <p>
                        <strong
                            >From: {msg.sender_npub} at {msg.timestamp}</strong
                        >
                    </p>
                    <div>{@html sanitizeHtml(msg.text)}</div>
                </div>
            {/each}
        </div>
        <div>
            <button on:click={goToCompose}>Compose New Message</button>
        </div>
    {/if}
</main>

<style>
    main {
        padding: 1rem;
        max-width: 800px;
        margin: 0 auto;
    }
    .message {
        border-bottom: 1px solid #eee;
        margin-bottom: 1rem;
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
        margin-top: 1rem;
    }
    button:hover {
        background-color: #0056b3;
    }
</style>
