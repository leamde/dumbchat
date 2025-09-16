<script>
    import { onMount, afterUpdate } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
    import { Editor } from "@tiptap/core";
    import StarterKit from "@tiptap/starter-kit";

    let recipientNostrPub = "";
    let recipientXPub = "";
    let editor = null;
    let editorElement = null;
    let message = "";
    let error = null;
    let loading = true;

    onMount(async () => {
        console.log("Compose page mounted");
        if (!window.__TAURI__) {
            error = "Tauri runtime not available";
            console.error("Tauri runtime not available");
            loading = false;
            return;
        }
        loading = false;
    });

    afterUpdate(() => {
        if (editorElement && !editor) {
            console.log("Initializing TipTap editor...");
            try {
                editor = new Editor({
                    element: editorElement,
                    extensions: [StarterKit],
                    content: "",
                    onUpdate: ({ editor }) => {
                        message = editor.getHTML();
                        console.log("Editor updated, message:", message);
                    },
                });
                console.log("TipTap editor initialized");
            } catch (editorError) {
                error = `Editor initialization failed: ${editorError.message || editorError}`;
                console.error("Editor error:", editorError);
            }
        }
    });

    async function sendMessage() {
        if (!recipientNostrPub || !recipientXPub || !message) {
            error = "Fill all fields";
            console.error("Send error: Missing fields");
            return;
        }
        try {
            console.log("Sending message...");
            const response = await invoke("send_nostr_message", {
                recipient_nostr_pub: recipientNostrPub,
                recipient_x_pub: recipientXPub,
                text: message,
            });
            console.log("Send response:", response);
            if (response.success) {
                editor?.commands.clearContent();
                message = "";
                error = null;
                console.log("Message sent successfully");
                goto("/inbox"); // Return to inbox after sending
            } else {
                error = response.message;
                console.error("Send error:", response.message);
            }
        } catch (err) {
            error = `Send failed: ${err.message || err}`;
            console.error("Send error:", err);
        }
    }

    function goToInbox() {
        console.log("Navigating to inbox");
        goto("/inbox");
    }
</script>

<main>
    <h1>Compose Message</h1>
    {#if loading}
        <p>Loading...</p>
    {/if}
    {#if error}
        <p style="color: red;">{error}</p>
    {/if}
    {#if !loading && !error}
        <div>
            <h2>Send Message</h2>
            <input
                bind:value={recipientNostrPub}
                placeholder="Recipient Nostr Pub (npub...)"
            />
            <input
                bind:value={recipientXPub}
                placeholder="Recipient X25519 Pub (hex)"
            />
            <div class="editor" bind:this={editorElement}></div>
            <button on:click={sendMessage}>Send</button>
            <button on:click={goToInbox}>Back to Inbox</button>
        </div>
    {/if}
</main>

<style>
    main {
        padding: 1rem;
        max-width: 800px;
        margin: 0 auto;
    }
    .editor {
        border: 1px solid #ccc;
        padding: 1rem;
        min-height: 200px;
        margin-bottom: 1rem;
    }
    input {
        padding: 0.8rem;
        font-size: 1.2rem;
        border: 1px solid #ccc;
        border-radius: 5px;
        width: 100%;
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
        margin-right: 1rem;
    }
    button:hover {
        background-color: #0056b3;
    }
</style>
