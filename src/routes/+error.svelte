<script>
    import { page } from "$app/stores";

    $: error = $page.error || {};
    $: status = $page.status || "Unknown";
</script>

<h1>Error: {status}</h1>
<p>Message: {error.message || "No message provided"}</p>
<p>Details: {error.stack || "No stack trace available"}</p>
<div>
    <h3>Error Object:</h3>
    <pre>{JSON.stringify(error, null, 2)}</pre>
</div>
<p>Status: {status}</p>
<div>
    <h3>Page Data:</h3>
    <pre>{JSON.stringify($page, null, 2)}</pre>
</div>

<style>
    h1 {
        color: red;
    }
    p,
    h3 {
        white-space: pre-wrap;
    }
    pre {
        background-color: #f4f4f4;
        padding: 1rem;
        border-radius: 5px;
        overflow-x: auto;
    }
    div {
        margin-bottom: 1rem;
    }
</style>
