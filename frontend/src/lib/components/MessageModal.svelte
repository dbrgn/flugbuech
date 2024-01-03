<script lang="ts">
  import {createEventDispatcher} from 'svelte';

  export let type: 'warning' | 'error';
  export let title: string;
  export let message: string;
  export let showClose = true;

  $: articleClass = type === 'warning' ? 'is-warning' : 'is-danger';

  const dispatch = createEventDispatcher<{closed: void}>();
</script>

<div class="modal is-active">
  <div class="modal-background"></div>
  <div class="modal-content">
    <article class="message {articleClass}">
      <div class="message-header">
        <p>{title}</p>
        {#if showClose}
          <button class="delete" aria-label="close" on:click={() => dispatch('closed', undefined)}
          ></button>
        {/if}
      </div>
      <div class="message-body" class:has-buttons={$$slots.buttons}>
        <section>{message}</section>
        <slot name="buttons" />
      </div>
    </article>
  </div>
</div>

<style>
  /* TODO: Upgrade Bulma to 0.9+ and use the spacing helpers. */
  .message-body.has-buttons > section:first-of-type {
    margin-bottom: 16px;
  }
</style>
