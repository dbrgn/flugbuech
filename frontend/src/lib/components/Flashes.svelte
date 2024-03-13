<script lang="ts">
  import {flashes, type Flash} from '$lib/stores';
  import {onMount} from 'svelte';

  let loadedFlashes: Flash[] = [];

  export function update(replace = false): void {
    if ($flashes.length > 0) {
      loadedFlashes = replace ? $flashes : [...loadedFlashes, ...$flashes];
      $flashes = [];
    }
  }

  onMount(() => {
    update(false);
  });
</script>

{#if loadedFlashes.length > 0}
  <div class="container flash-container">
    {#each loadedFlashes as flash}
      <article
        class="message"
        class:is-info={flash.severity === 'info'}
        class:is-success={flash.severity === 'success'}
        class:is-warning={flash.severity === 'warning'}
        class:is-danger={flash.severity === 'error'}
      >
        <div class="message-body">
          {#if flash.icon}<i class="fa-solid {flash.icon}" />&nbsp;{/if}
          {flash.message}
        </div>
      </article>
    {/each}
  </div>
{/if}
