<script>
    import { onMount } from "svelte";
    import { invoke, listen } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
    import QRCode from "qrcode";
    import sanitizeHtml from "sanitize-html";

    let nostrPub = "";
    let xPub = "";
    let messages = [];
    let error = null;
    let loading = true;

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
            console.log("Setting up new_message listener...");
            const unlisten = await listen("new_message", (event) => {
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

            // No invoke calls to isolate routing issue
            nostrPub = "Not loaded (invoke skipped)";
            xPub = "Not loaded (invoke skipped)";
            console.log("Skipping get_user_info to test page load");

            await new Promise((resolve) => setTimeout(resolve, 100));
            const nostrCanvas = document.getElementById("nostr-qr");
            const xCanvas = document.getElementById("x-qr");
            if (nostrCanvas && xCanvas) {
                try {
                    console.log("Rendering placeholder QR codes...");
                    await QRCode.toCanvas(nostrCanvas, nostrPub, { scale: 4 });
                    await QRCode.toCanvas(xCanvas, xPub, { scale: 4 });
                    console.log("QR codes rendered successfully");
                } catch (qrError) {
                    error = `QR code rendering failed: ${qrError.message || qrError}`;
                    console.error(
                        "QR code error:",
                        JSON.stringify(qrError, null, 2),
                    );
                    loading = false;
                    return unlisten;
                }
            } else {
                error = "Canvas elements not found";
                console.error(
                    "Canvas error: nostrCanvas=",
                    !!nostrCanvas,
                    "xCanvas=",
                    !!xCanvas,
                );
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
            <h2>Your Keys (Share these to chat)</h2>
            <p>Nostr Pub: {nostrPub || "Not available"}</p>
            <canvas id="nostr-qr"></canvas>
            <p>X25519 Pub: {xPub || "Not available"}</p>
            <canvas id="x-qr"></canvas>
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
