<script lang="ts">
  import {goto, invalidateAll} from '$app/navigation';
  import {apiDelete, extractResponseError} from '$lib/api';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {addFlash} from '$lib/stores';
  import {formatDuration} from '$lib/formatters';
  import type {Data} from './+page';
  import type {Glider} from './api';

  export let data: Data;

  let flashes: Flashes;

  let gliderToDelete: Glider | undefined;
  let deleting = false;
  let deleteError: {type: 'authentication'} | {type: 'api-error'; message: string} | undefined;

  async function deleteGlider(): Promise<void> {
    if (gliderToDelete !== undefined) {
      deleting = true;
      console.info(`Deleting glider with ID ${gliderToDelete.id}`);

      const response = await apiDelete(`/api/v1/gliders/${gliderToDelete.id}`);
      switch (response.status) {
        case 204:
          // Success
          addFlash({
            message: `Glider "${gliderToDelete.manufacturer} ${gliderToDelete.model}" successfully deleted`,
            severity: 'success',
            icon: 'fa-trash-can',
          });
          goto('/gliders/');
          break;
        case 401:
          deleteError = {type: 'authentication'};
          break;
        default: {
          deleteError = {
            type: 'api-error',
            message: await extractResponseError(response),
          };
          break;
        }
      }

      // Show flash message
      flashes.update(true);

      // Hide delete dialog
      gliderToDelete = undefined;
      deleting = false;

      // Reload data
      invalidateAll();
    }
  }
</script>

{#if deleteError?.type === 'authentication'}
  <MessageModal
    type="warning"
    title="Authentication Error"
    message="Your login session has expired. Please log in again."
    showClose={false}
  >
    <section slot="buttons">
      <a href="/auth/login/" class="button is-warning">Login</a>
    </section>
  </MessageModal>
{:else if deleteError?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message="The glider could not be deleted due to an error on the server: {deleteError.message}"
    showClose={true}
    on:closed={() => (deleteError = undefined)}
  />
{:else if gliderToDelete !== undefined}
  <DialogModal
    title="Delete Glider"
    message="Are you sure that you want to delete the glider &ldquo;{gliderToDelete.manufacturer} {gliderToDelete.model}&rdquo;?"
    dialogClass="is-danger"
  >
    <section slot="buttons">
      <button class="button is-light" on:click={() => (gliderToDelete = undefined)}
        >No, cancel</button
      >
      <button class="button is-danger" disabled={deleting} on:click={() => void deleteGlider()}
        >Yes, delete!</button
      >
    </section>
  </DialogModal>
{/if}

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Gliders</a></li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">Your Gliders</h2>

<section>
  <p class="content">
    You flew {data.gliders.length} glider{data.gliders.length === 1 ? '' : 's'} so far!
  </p>
  <p class="content">
    <a href="/gliders/add/" class="button is-info">Add glider</a>
  </p>
  <table class="table is-fullwidth is-striped is-hoverable">
    <thead>
      <tr>
        <th>Manufacturer</th>
        <th>Model</th>
        <th>Since</th>
        <th>Until</th>
        <th>Flights</th>
        <th>Hours</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each data.gliders as glider (glider.id)}
        <tr>
          <td>{glider.manufacturer}</td>
          <td>{glider.model}</td>
          <td>
            {#if glider.since !== undefined}{glider.since}{:else}-{/if}
          </td>
          <td>
            {#if glider.until !== undefined}{glider.until}{:else}-{/if}
          </td>
          <td>
            {#if glider.stats.flights > 0}{glider.stats.flights}{:else}-{/if}
          </td>
          <td>
            {#if glider.stats.seconds > 0}{formatDuration(
                glider.stats.seconds,
              )}{:else}-{/if}{#if !glider.stats.secondsComplete}&nbsp;<sup>1</sup>{/if}
          </td>
          <td>
            <a
              class="icon"
              title="Edit Glider"
              href="/gliders/{glider.id}/edit/"
              data-sveltekit-preload-data="tap"
            >
              <i class="fas fa-pen-square"></i>
            </a>
            {#if glider.stats.flights === 0}
              <button
                class="icon has-text-danger"
                title="Delete Glider"
                on:click={() => (gliderToDelete = glider)}
              >
                <i class="fa-solid fa-trash-alt"></i>
              </button>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>

{#if data.gliders.some((glider) => !glider.stats.secondsComplete)}
  <p class="incomplete-warning">
    <small>
      <sup>1</sup> Warning: There are flights without launch date/time in your flight book, these will
      not contribute towards the glider hour stats.
    </small>
  </p>
{/if}

<style>
  table button {
    background-color: transparent;
    border: none;
    cursor: pointer;
  }

  table button:hover i {
    color: #c41e1e;
  }

  .incomplete-warning {
    margin-top: 16px;
  }
</style>
