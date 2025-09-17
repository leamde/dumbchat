<script>
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";

    let recipientNostrPub = "";
    let recipientXPub = "";
    let editor = null;
    let error = null;
    let tauriCore = null;
    let selectedFont = "Arial";
    let selectedFontSize = "16px";
    let selectedColor = "#000000";
    let selectedHighlight = "#ffff00";
    let selectedAlignment = "left";

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

            console.log("Dynamically importing TipTap modules...");
            const { Editor } = await import("@tiptap/core");
            console.log("Imported Editor");
            const StarterKit = await import("@tiptap/starter-kit");
            console.log("Imported StarterKit");
            const Placeholder = await import("@tiptap/extension-placeholder");
            console.log("Imported Placeholder");
            const TextStyle = await import("@tiptap/extension-text-style");
            console.log("Imported TextStyle");
            const Color = await import("@tiptap/extension-color");
            console.log("Imported Color");
            const Highlight = await import("@tiptap/extension-highlight");
            console.log("Imported Highlight");
            const Underline = await import("@tiptap/extension-underline");
            console.log("Imported Underline");
            const Strike = await import("@tiptap/extension-strike");
            console.log("Imported Strike");
            const BulletList = await import("@tiptap/extension-bullet-list");
            console.log("Imported BulletList");
            const OrderedList = await import("@tiptap/extension-ordered-list");
            console.log("Imported OrderedList");
            const ListItem = await import("@tiptap/extension-list-item");
            console.log("Imported ListItem");
            const Table = await import("@tiptap/extension-table");
            console.log("Imported Table");
            const TableCell = await import("@tiptap/extension-table-cell");
            console.log("Imported TableCell");
            const TableHeader = await import("@tiptap/extension-table-header");
            console.log("Imported TableHeader");
            const TableRow = await import("@tiptap/extension-table-row");
            console.log("Imported TableRow");
            const Link = await import("@tiptap/extension-link");
            console.log("Imported Link");
            const CodeBlock = await import("@tiptap/extension-code-block");
            console.log("Imported CodeBlock");
            const Typography = await import("@tiptap/extension-typography");
            console.log("Imported Typography");
            console.log("TipTap modules imported successfully");

            const editorElement = document.querySelector(".editor");
            if (!editorElement) {
                error = "Editor element not found";
                console.error("Editor element not found");
                return;
            }

            editor = new Editor({
                element: editorElement,
                extensions: [
                    StarterKit.default.configure({
                        heading: { levels: [1, 2, 3] },
                    }),
                    Placeholder.default.configure({
                        placeholder: "Type your message here...",
                    }),
                    TextStyle.default,
                    Color.default,
                    Highlight.default.configure({ multicolor: true }),
                    Underline.default,
                    Strike.default,
                    BulletList.default,
                    OrderedList.default,
                    ListItem.default,
                    Table.default.configure({
                        resizable: true,
                    }),
                    TableCell.default,
                    TableHeader.default,
                    TableRow.default,
                    Link.default.configure({
                        openOnClick: false,
                    }),
                    CodeBlock.default,
                    Typography.default,
                ],
                content: "",
                onUpdate: ({ editor }) => {
                    console.log("Editor updated, content:", editor.getHTML());
                },
            });
            console.log(
                "TipTap editor initialized with full suite of features",
            );
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

    function setBold() {
        if (editor) {
            editor.chain().focus().toggleBold().run();
            console.log("Toggled bold, content:", editor.getHTML());
        }
    }

    function setItalic() {
        if (editor) {
            editor.chain().focus().toggleItalic().run();
            console.log("Toggled italic, content:", editor.getHTML());
        }
    }

    function setUnderline() {
        if (editor) {
            editor.chain().focus().toggleUnderline().run();
            console.log("Toggled underline, content:", editor.getHTML());
        }
    }

    function setStrikethrough() {
        if (editor) {
            editor.chain().focus().toggleStrike().run();
            console.log("Toggled strikethrough, content:", editor.getHTML());
        }
    }

    function setTextColor(color) {
        if (editor) {
            editor.chain().focus().setColor(color).run();
            console.log("Set text color to:", color);
        }
    }

    function setHighlight(color) {
        if (editor) {
            editor.chain().focus().toggleHighlight({ color }).run();
            console.log("Set highlight to:", color);
        }
    }

    function setFontFamily(font) {
        if (editor) {
            selectedFont = font;
            editor.chain().focus().setFontFamily(font).run();
            console.log("Font changed to:", font);
        }
    }

    function setFontSize(size) {
        if (editor) {
            selectedFontSize = size;
            editor.chain().focus().setFontSize(size).run();
            console.log("Font size changed to:", size);
        }
    }

    function setAlignment(alignment) {
        if (editor) {
            selectedAlignment = alignment;
            editor.chain().focus().setTextAlign(alignment).run();
            console.log("Alignment changed to:", alignment);
        }
    }

    function toggleBulletList() {
        if (editor) {
            editor.chain().focus().toggleBulletList().run();
            console.log("Toggled bullet list, content:", editor.getHTML());
        }
    }

    function toggleOrderedList() {
        if (editor) {
            editor.chain().focus().toggleOrderedList().run();
            console.log("Toggled ordered list, content:", editor.getHTML());
        }
    }

    function insertTable() {
        if (editor) {
            editor
                .chain()
                .focus()
                .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
                .run();
            console.log("Inserted table, content:", editor.getHTML());
        }
    }

    function addLink() {
        if (editor) {
            const url = prompt("Enter link URL:");
            if (url) {
                editor.chain().focus().setLink({ href: url }).run();
                console.log("Added link, content:", editor.getHTML());
            }
        }
    }

    function toggleCodeBlock() {
        if (editor) {
            editor.chain().focus().toggleCodeBlock().run();
            console.log("Toggled code block, content:", editor.getHTML());
        }
    }

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
        <div class="toolbar">
            <button on:click={setBold}>Bold</button>
            <button on:click={setItalic}>Italic</button>
            <button on:click={setUnderline}>Underline</button>
            <button on:click={setStrikethrough}>Strikethrough</button>
            <select
                bind:value={selectedColor}
                on:change={() => setTextColor(selectedColor)}
            >
                <option value="#000000">Black</option>
                <option value="#ff0000">Red</option>
                <option value="#0000ff">Blue</option>
            </select>
            <select
                bind:value={selectedHighlight}
                on:change={() => setHighlight(selectedHighlight)}
            >
                <option value="#ffff00">Yellow</option>
                <option value="#90ee90">Green</option>
                <option value="#add8e6">Blue</option>
            </select>
            <select
                bind:value={selectedFont}
                on:change={() => setFontFamily(selectedFont)}
            >
                <option value="Arial">Arial</option>
                <option value="Courier">Courier</option>
                <option value="Times New Roman">Times New Roman</option>
            </select>
            <select
                bind:value={selectedFontSize}
                on:change={() => setFontSize(selectedFontSize)}
            >
                <option value="12px">12px</option>
                <option value="16px">16px</option>
                <option value="20px">20px</option>
            </select>
            <select
                bind:value={selectedAlignment}
                on:change={() => setAlignment(selectedAlignment)}
            >
                <option value="left">Left</option>
                <option value="center">Center</option>
                <option value="right">Right</option>
            </select>
            <button on:click={toggleBulletList}>Bullet List</button>
            <button on:click={toggleOrderedList}>Numbered List</button>
            <button on:click={insertTable}>Insert Table</button>
            <button on:click={addLink}>Add Link</button>
            <button on:click={toggleCodeBlock}>Code Block</button>
        </div>
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
    .toolbar {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
        margin-bottom: 1rem;
    }
    .editor {
        border: 1px solid #ccc;
        padding: 1rem;
        min-height: 200px;
        margin-bottom: 1rem;
        font-family: Arial, sans-serif;
    }
    button,
    select {
        padding: 0.5rem 1rem;
        font-size: 1rem;
        background-color: #007bff;
        color: white;
        border: none;
        border-radius: 5px;
        cursor: pointer;
        transition: background-color 0.2s;
    }
    button:hover,
    select:hover {
        background-color: #0056b3;
    }
</style>
