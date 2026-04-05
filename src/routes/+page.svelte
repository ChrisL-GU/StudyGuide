<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  onMount(() => {
    goto('/documents');
  });
</script>

<p>Redirecting...</p>
