<script lang="ts">
  import {notUndefined} from '$lib';
  import {apiDelete, extractResponseError} from '$lib/api';
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import MultiMap from '$lib/components/MultiMap.svelte';
  import {i18n} from '$lib/i18n';
  import {addFlash} from '$lib/stores';

  import {goto, invalidateAll} from '$app/navigation';

  import type {Data} from './+page';
  import type {Location} from './api';

  export let data: Data;

  let flashes: Flashes;

  let locationToDelete: Location | undefined;
  let deleting = false;
  let deleteError: {type: 'authentication'} | {type: 'api-error'; message: string} | undefined;

  async function deleteLocation(): Promise<void> {
    if (locationToDelete !== undefined) {
      deleting = true;
      console.info(`Deleting location with ID ${locationToDelete.id}`);

      const response = await apiDelete(`/api/v1/locations/${locationToDelete.id}`);
      switch (response.status) {
        case 204:
          // Success
          addFlash({
            message: $i18n.t('locations.prose--delete-success', {
              name: locationToDelete.name,
            }),
            severity: 'success',
            icon: 'fa-trash-can',
          });
          goto('/locations/');
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
      locationToDelete = undefined;
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
    message={$i18n.t('locations.prose--delete-error', {message: deleteError.message})}
    showClose={true}
    on:closed={() => (deleteError = undefined)}
  />
{:else if locationToDelete !== undefined}
  <DialogModal
    title={$i18n.t('locations.action--delete-location')}
    message={$i18n.t('locations.prose--delete-confirm', {name: locationToDelete.name})}
    dialogClass="is-danger"
  >
    <section slot="buttons">
      <button class="button is-white" on:click={() => (locationToDelete = undefined)}>
        {$i18n.t('common.action--no-cancel')}
      </button>
      <button class="button is-danger" disabled={deleting} on:click={() => void deleteLocation()}>
        {$i18n.t('common.action--yes-delete')}
      </button>
    </section>
  </DialogModal>
{/if}

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">{$i18n.t('navigation.locations')}</a>
    </li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">{$i18n.t('locations.title--your-locations')}</h2>

<section>
  <article class="message is-info">
    <div class="message-body">
      <i class="fa-solid fa-circle-info"></i>&ensp;
      {$i18n.t('locations.prose--hint-nonglobal')}
    </div>
  </article>

  <p class="mb-4">
    {$i18n.t('locations.prose--location-count', {count: data.locations.length})}
  </p>
  <div class="mb-5">
    <a href="/locations/add/" class="button is-primary">
      {$i18n.t('locations.action--add-location')}
    </a>
  </div>
  {#if data.locations.some((location) => location.coordinates !== undefined)}
    <section class="map mb-5">
      <MultiMap
        markers={data.locations
          .map((location) =>
            location.coordinates === undefined
              ? undefined
              : {name: location.name, ...location.coordinates},
          )
          .filter(notUndefined)}
      />
    </section>
  {/if}
  <div class="table-container">
    <table class="table is-fullwidth is-striped is-hoverable">
      <thead>
        <tr>
          <th>{$i18n.t('locations.column--name')}</th>
          <th>{$i18n.t('locations.column--country')}</th>
          <th>{$i18n.t('locations.column--elevation')}</th>
          <th>{$i18n.t('locations.column--flights')}</th>
          <th>{$i18n.t('locations.column--actions')}</th>
        </tr>
      </thead>
      <tbody>
        {#each data.locations as location (location.id)}
          <tr>
            <td>{location.name}</td>
            <td class="no-wrap">
              <CountryFlag countryCode={location.countryCode} />
              {location.countryCode}
            </td>
            <td class="no-wrap">{location.elevation} {$i18n.t('common.unit--m-asl')}</td>
            <td class="no-wrap">{location.flightCount}</td>
            <td class="no-wrap">
              <a
                class="icon"
                title={$i18n.t('locations.action--view-location')}
                aria-label={$i18n.t('locations.action--view-location')}
                href="/locations/{location.id}/"
              >
                <i class="fa-solid fa-eye"></i>
              </a>
              <a
                class="icon"
                title={$i18n.t('locations.action--edit-location')}
                aria-label={$i18n.t('locations.action--edit-location')}
                href="/locations/{location.id}/edit/"
                data-sveltekit-preload-data="tap"
              >
                <i class="fa-solid fa-pen-square"></i>
              </a>
              {#if location.flightCount === 0}
                <button
                  class="icon has-text-danger"
                  title={$i18n.t('locations.action--delete-location')}
                  aria-label={$i18n.t('locations.action--delete-location')}
                  on:click={() => (locationToDelete = location)}
                >
                  <i class="fa-solid fa-trash-alt"></i>
                </button>
              {/if}
              {#if location.coordinates !== undefined}
                {@const lon = location.coordinates.lon}
                {@const lat = location.coordinates.lat}
                <a
                  class="icon"
                  href="https://www.google.com/maps/place/{lat},{lon}/"
                  title={$i18n.t('locations.action--view-google-maps')}
                  aria-label={$i18n.t('locations.action--view-google-maps')}
                  target="_blank"
                >
                  <i class="fa-solid fa-map-marker-alt"></i>
                </a>
                <a
                  class="icon"
                  href="https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=16/{lat}/{lon}"
                  title={$i18n.t('locations.action--view-osm')}
                  aria-label={$i18n.t('locations.action--view-osm')}
                  target="_blank"
                >
                  <i class="fa-solid fa-map-pin"></i>
                </a>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>

<style>
  table button {
    background-color: transparent;
    border: none;
    cursor: pointer;
  }

  table button:hover i {
    color: #c41e1e;
  }
</style>
