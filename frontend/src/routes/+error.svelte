<script lang="ts">
  import {page} from '$app/stores';

  import Flashes from '$lib/components/Flashes.svelte';
</script>

<Flashes />

<h2 class="title is-size-2">Oh no!</h2>

<p class="content">An error occurred:</p>

<article class="message is-danger">
  <div class="message-body">
    <p class="content">
      {$page.error?.message ?? 'Unknown error'}
    </p>

    <p class="content">Error code: {$page.status}</p>
  </div>
</article>
