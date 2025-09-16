<script>
    import { beforeNavigate, afterNavigate } from "$app/navigation";
    import { onMount } from "svelte";

    onMount(() => {
        console.log("Layout mounted at", new Date().toISOString());
    });

    beforeNavigate(({ to, from }) => {
        console.log(
            "Before navigation - from:",
            from?.url?.toString() || "unknown",
            "to:",
            to?.url?.toString() || "unknown",
        );
    });

    afterNavigate(({ to, from }) => {
        console.log(
            "After navigation - from:",
            from?.url?.toString() || "unknown",
            "to:",
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
