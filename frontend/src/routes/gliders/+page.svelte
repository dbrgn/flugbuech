<script lang="ts">
  import {apiDelete, extractResponseError} from '$lib/api';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {formatDuration} from '$lib/formatters';
  import {i18n} from '$lib/i18n';
  import {addFlash} from '$lib/stores';

  import {goto, invalidateAll} from '$app/navigation';

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
            message: $i18n.t('gliders.prose--delete-success', {
              manufacturer: gliderToDelete.manufacturer,
              model: gliderToDelete.model,
            }),
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
    title={$i18n.t('common.error--authentication-error')}
    message={$i18n.t('common.error--login-session-expired')}
    showClose={false}
  >
    <section slot="buttons">
      <a href="/auth/login/" class="button is-warning">{$i18n.t('navigation.login')}</a>
    </section>
  </MessageModal>
{:else if deleteError?.type === 'api-error'}
  <MessageModal
    type="error"
    title={$i18n.t('common.error--api-error')}
    message={$i18n.t('gliders.prose--delete-error', {message: deleteError.message})}
    showClose={true}
    on:closed={() => (deleteError = undefined)}
  />
{:else if gliderToDelete !== undefined}
  <DialogModal
    title={$i18n.t('gliders.action--delete-glider')}
    message={$i18n.t(
      'gliders.prose--delete-confirm',
      'Are you sure that you want to delete the glider «{manufacturer} {model}»?',
      {
        manufacturer: gliderToDelete.manufacturer,
        model: gliderToDelete.model,
      },
    )}
    dialogClass="is-danger"
  >
    <section slot="buttons">
      <button class="button is-white" on:click={() => (gliderToDelete = undefined)}>
        {$i18n.t('common.action--no-cancel')}
      </button>
      <button class="button is-danger" disabled={deleting} on:click={() => void deleteGlider()}>
        {$i18n.t('common.action--yes-delete')}
      </button>
    </section>
  </DialogModal>
{/if}

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li class="is-active"><a href="./" aria-current="page">{$i18n.t('navigation.gliders')}</a></li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">{$i18n.t('gliders.title--your-gliders')}</h2>

<section>
  <p class="content">
    {$i18n.t('gliders.prose--glider-count', {count: data.gliders.length})}
  </p>
  <p class="content">
    <a href="/gliders/add/" class="button is-primary">
      {$i18n.t('gliders.action--add-glider')}
    </a>
  </p>
  <table class="table is-fullwidth is-striped is-hoverable">
    <thead>
      <tr>
        <th>{$i18n.t('gliders.column--manufacturer')}</th>
        <th>{$i18n.t('gliders.column--model')}</th>
        <th>{$i18n.t('gliders.column--since')}</th>
        <th>{$i18n.t('gliders.column--until')}</th>
        <th>{$i18n.t('gliders.column--flights')}</th>
        <th>{$i18n.t('gliders.column--hours')}</th>
        <th>{$i18n.t('gliders.column--actions')}</th>
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
              title={$i18n.t('gliders.action--edit-glider')}
              aria-label={$i18n.t('gliders.action--edit-glider')}
              href="/gliders/{glider.id}/edit/"
              data-sveltekit-preload-data="tap"
            >
              <i class="fas fa-pen-square"></i>
            </a>
            {#if glider.stats.flights === 0}
              <button
                class="icon has-text-danger"
                title={$i18n.t('gliders.action--delete-glider')}
                aria-label={$i18n.t('gliders.action--delete-glider')}
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
      <sup>1</sup>
      {$i18n.t('gliders.prose--warning-incomplete')}
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
