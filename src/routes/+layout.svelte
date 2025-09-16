<script>
    import { beforeNavigate } from "$app/navigation";
    import { onMount } from "svelte";

    onMount(() => {
        console.log("Layout mounted at", new Date().toISOString());
        // Try to open devtools manually
        if (window.__TAURI__) {
            console.log("Attempting to open devtools in layout...");
            window.__TAURI__.window
                .getCurrent()
                .openDevtools()
                .catch((err) => {
                    console.error(
                        "Failed to open devtools in layout:",
                        JSON.stringify(err, null, 2),
                    );
                });
        }
    });

    beforeNavigate(({ to, from }) => {
        console.log(
            "Navigating from",
            from?.url?.toString() || "unknown",
            "to",
            to?.url?.toString() || "unknown",
        );
    });
</script>

<slot />

<style>
    :global(body) {
        margin: 0;
        font-family: Arial, sans-serif;
    }
</style>
