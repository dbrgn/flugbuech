<script lang="ts">
  import {invalidateAll} from '$app/navigation';
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import DialogModal from '$lib/components/DialogModal.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import {addFlash} from '$lib/stores';
  import type {Data} from './+page';
  import type {Location} from './api';

  export let data: Data;

  let flashes: Flashes;

  let locationToDelete: Location | undefined;

  function deleteLocation(): void {
    if (locationToDelete !== undefined) {
      console.info(`Deleting location with ID ${locationToDelete.id}`);

      // TODO actually delete

      // Show flash success message
      addFlash({
        message: `Location "${locationToDelete.name}" successfully deleted!`,
        severity: 'success',
        icon: 'fa-trash-can',
      });
      flashes.update();

      // Hide delete dialog
      locationToDelete = undefined;

      // Reload data
      invalidateAll();
    }
  }
</script>

{#if locationToDelete !== undefined}
  <DialogModal
    title="Delete Location"
    message="Are you sure that you want to delete the location &ldquo;{locationToDelete.name}&rdquo;?"
    dialogClass="is-danger"
  >
    <section slot="buttons">
      <button class="button is-light" on:click={() => (locationToDelete = undefined)}
        >No, cancel</button
      >
      <button class="button is-danger" on:click={() => deleteLocation()}>Yes, delete!</button>
    </section>
  </DialogModal>
{/if}

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Locations</a></li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">Your Locations</h2>

<section>
  <article class="message is-info">
    <div class="message-body">
      <i class="fa-solid fa-circle-info" />&emsp;Note: A location can be used both as launch
      location and as landing location. Locations are not global, i.e. you are creating and
      maintaining your own location database.
    </div>
  </article>

  <p class="content">
    You've been at {data.locations.length} location{data.locations.length === 1 ? '' : 's'} so far!
  </p>
  <p class="content">
    <a href="/locations/add/" class="button is-primary">Add location</a>
  </p>
  <table class="table is-fullwidth is-striped is-hoverable">
    <thead>
      <tr>
        <th>Name</th>
        <th>Country</th>
        <th>Elevation</th>
        <th>Flights</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each data.locations as location (location.id)}
        <tr>
          <td>{location.name}</td>
          <td><CountryFlag countryCode={location.countryCode} /> {location.countryCode}</td>
          <td>{location.elevation} m ASL</td>
          <td>{location.flightCount}</td>
          <td>
            <a class="icon" title="View Location" href="/locations/{location.id}">
              <i class="fa-solid fa-eye"></i>
            </a>
            <a class="icon" title="Edit Location" href="/locations/{location.id}/edit/">
              <i class="fa-solid fa-pen-square"></i>
            </a>
            {#if location.flightCount === 0}
              <button
                class="icon has-text-danger"
                title="Delete Location"
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
                title="View in Google Maps"
              >
                <i class="fa-solid fa-map-marker-alt"></i>
              </a>
              <a
                class="icon"
                href="https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=16/{lat}/{lon}"
                title="View in OpenStreetMap"
              >
                <i class="fa-solid fa-map-pin"></i>
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
