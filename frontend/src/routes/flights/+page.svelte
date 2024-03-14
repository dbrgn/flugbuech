<script lang="ts">
  import {apiDelete, extractResponseError} from '$lib/api';
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import XContestSummary from '$lib/components/XContestSummary.svelte';
  import {formatDate, formatDistance, formatDuration} from '$lib/formatters';
  import {addFlash} from '$lib/stores';

  import {goto, invalidateAll} from '$app/navigation';

  import type {Data} from './+page';
  import type {FlightListItem} from './api';

  export let data: Data;

  let flashes: Flashes;

  let flightToDelete: FlightListItem | undefined;
  let deleting = false;
  let deleteError: {type: 'authentication'} | {type: 'api-error'; message: string} | undefined;

  function flightName(flight: FlightListItem): string {
    let name = `Flight ${
      flight.number !== undefined ? `${flight.number}` : `with ID ${flight.id}`
    }`;
    if (flight.launchAt !== undefined) {
      const location = data.locations[flight.launchAt];
      if (location !== undefined) {
        name += ` from ${location.name}`;
      }
    }
    if (flight.landingAt !== undefined) {
      const location = data.locations[flight.landingAt];
      if (location !== undefined) {
        name += ` to ${location.name}`;
      }
    }
    return name;
  }

  async function deleteFlight(): Promise<void> {
    if (flightToDelete !== undefined) {
      deleting = true;
      console.info(`Deleting flight with ID ${flightToDelete.id}`);

      const response = await apiDelete(`/api/v1/flights/${flightToDelete.id}/`);
      switch (response.status) {
        case 204:
          // Success
          addFlash({
            message: `${flightName(flightToDelete)} successfully deleted`,
            severity: 'success',
            icon: 'fa-trash-can',
          });
          goto('/flights/');
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
      flightToDelete = undefined;
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
    message="The flight could not be deleted due to an error on the server: {deleteError.message}"
    showClose={true}
    on:closed={() => (deleteError = undefined)}
  />
{:else if flightToDelete !== undefined}
  <DialogModal
    title="Delete Flight"
    message="Are you sure that you want to delete the flight &ldquo;{flightName(
      flightToDelete,
    )}&rdquo;?"
    dialogClass="is-danger"
  >
    <section slot="buttons">
      <button class="button is-light" on:click={() => (flightToDelete = undefined)}
        >No, cancel</button
      >
      <button class="button is-danger" disabled={deleting} on:click={() => void deleteFlight()}
        >Yes, delete!</button
      >
    </section>
  </DialogModal>
{/if}

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Flights</a></li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">Your Flights</h2>

<section>
  <p class="content">
    You've logged {data.flights.length} flight{data.flights.length === 1 ? '' : 's'} so far!
  </p>
  <p class="content">
    <a href="/flights/add/" class="button is-info">Add flight</a>
  </p>
  <table class="table is-fullwidth is-striped is-hoverable">
    <thead>
      <tr>
        <th>#</th>
        <th>Date</th>
        <th>Glider</th>
        <th>Launch</th>
        <th>Landing</th>
        <th>Duration</th>
        <th>GPS Distance</th>
        <th>XContest</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each data.flights as flight (flight.id)}
        {@const launchAt = flight.launchAt ? data.locations[flight.launchAt] : undefined}
        {@const landingAt = flight.landingAt ? data.locations[flight.landingAt] : undefined}
        <tr>
          <td>{flight.number ?? '-'}</td>
          <td>
            {#if flight.launchTime !== undefined}{formatDate(flight.launchTime)}{:else}-{/if}
          </td>
          <td>{flight.gliderName ?? '-'}</td>
          <td title={launchAt !== undefined ? `${launchAt.elevation} mASL` : ''}>
            {#if launchAt}
              <CountryFlag countryCode={launchAt.countryCode} />
              <a class="subtle-link" href="/locations/{launchAt.id}">{launchAt.name}</a
              >{#if flight.hikeandfly}&nbsp;<i class="fa-solid fa-hiking" title="Hike &amp; Fly"
                ></i>{/if}
            {:else}
              -
            {/if}
          </td>
          <td title={landingAt !== undefined ? `${landingAt.elevation} mASL` : ''}>
            {#if landingAt}
              <CountryFlag countryCode={landingAt.countryCode} />
              <a class="subtle-link" href="/locations/{landingAt.id}">{landingAt.name}</a>
            {:else}
              -
            {/if}
          </td>
          <td>
            {#if flight.durationSeconds}{formatDuration(flight.durationSeconds)}{:else}-{/if}
          </td>
          <td>
            {#if flight.trackDistance}{formatDistance(flight.trackDistance)}{:else}-{/if}
          </td>
          <td>
            <XContestSummary
              tracktype={flight.xcontestTracktype}
              distance={flight.xcontestDistance}
              url={flight.xcontestUrl}
              subtleLink={true}
            />
          </td>
          <td>
            <a class="icon" title="View Flight" href="/flights/{flight.id}/">
              <i class="fa-solid fa-eye"></i>
            </a>
            <a
              class="icon"
              title="Edit Flight"
              href="/flights/{flight.id}/edit/"
              data-sveltekit-preload-data="tap"
            >
              <i class="fa-solid fa-pen-square"></i>
            </a>
            <button
              class="icon has-text-danger"
              title="Delete Flight"
              on:click={() => (flightToDelete = flight)}
            >
              <i class="fa-solid fa-trash-alt"></i>
            </button>
            {#if flight.hasIgc}
              <a
                class="icon"
                title="Download IGC"
                href="/api/v1/flights/{flight.id}/igc/"
                data-sveltekit-reload
              >
                <i class="fa-solid fa-download"></i>
              </a>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
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
