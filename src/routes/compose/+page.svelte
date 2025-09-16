<script>
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { Editor } from "@tiptap/core";
    import StarterKit from "@tiptap/starter-kit";

    let editor;
    let error = null;
    let recipientNostrPub = "";
    let recipientXPub = "";
    let tauriCore = null;

    onMount(async () => {
        console.log("Compose page mounted at", new Date().toISOString());
        await new Promise((resolve) => setTimeout(resolve, 100));
        if (!window.__TAURI__) {
            error = "Tauri runtime not available";
            console.error("Tauri runtime not available");
            return;
        }

        try {
            console.log("Dynamically importing @tauri-apps/api/core...");
            tauriCore = await import("@tauri-apps/api/core");
            console.log("Tauri core imported successfully");

            editor = new Editor({
                element: document.querySelector(".editor"),
                extensions: [StarterKit],
                content: "<p>Type your message here...</p>",
            });
            console.log("TipTap editor initialized");
        } catch (err) {
            error = `Compose initialization failed: ${err.message || err}`;
            console.error("Compose init error:", JSON.stringify(err, null, 2));
        }

        return () => {
            if (editor) {
                editor.destroy();
            }
        };
    });

    async function sendMessage() {
        if (!editor || !recipientNostrPub || !recipientXPub) {
            error = "Please provide recipient keys and ensure editor is loaded";
            console.error("Send message validation failed", {
                recipientNostrPub,
                recipientXPub,
            });
            return;
        }

        try {
            const text = editor.getHTML();
            console.log("Invoking send_nostr_message...", {
                recipientNostrPub,
                recipientXPub,
                text,
            });
            const response = await tauriCore.invoke("send_nostr_message", {
                recipientNostrPub,
                recipientXPub,
                text,
            });
            console.log(
                "send_nostr_message response:",
                JSON.stringify(response, null, 2),
            );
            if (response.success) {
                console.log("Message sent, navigating to inbox");
                window.location.href = "/inbox";
            } else {
                error = response.message;
                console.error("Send message error:", response.message);
            }
        } catch (err) {
            error = `Send message failed: ${err.message || err}`;
            console.error("Send message error:", JSON.stringify(err, null, 2));
        }
    }
</script>

<main>
    <h1>Compose Message</h1>
    {#if error}
        <p style="color: red;">{error}</p>
    {/if}
    <div>
        <input
            type="text"
            bind:value={recipientNostrPub}
            placeholder="Recipient Nostr Public Key"
        />
        <input
            type="text"
            bind:value={recipientXPub}
            placeholder="Recipient X25519 Public Key"
        />
        <div class="editor"></div>
        <button on:click={sendMessage}>Send Message</button>
    </div>
</main>

<style>
    main {
        padding: 1rem;
        max-width: 800px;
        margin: 0 auto;
    }
    input {
        padding: 0.8rem;
        font-size: 1.2rem;
        border: 1px solid #ccc;
        border-radius: 5px;
        margin-bottom: 1rem;
        width: 100%;
    }
    .editor {
        border: 1px solid #ccc;
        padding: 1rem;
        min-height: 200px;
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
    }
    button:hover {
        background-color: #0056b3;
    }
</style>
