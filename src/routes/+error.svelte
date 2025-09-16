<script>
    import { page } from "$app/stores";

    $: error = $page.error;
</script>

<h1>Error: {error.message}</h1>
<p>Details: {error.stack}</p>

<style>
    h1 {
        color: red;
    }
</style>
