<script lang="ts">
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import type {Data} from './+page';

  export let data: Data;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Locations</a></li>
  </ul>
</nav>

<Flashes />

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
            <a class="icon" title="View Location" href="/locations/{location.id}"
              ><i class="fa-solid fa-eye"></i></a
            >
            <a href="/locations/{location.id}/edit/">
              <span class="icon"><i class="fa-solid fa-pen-square"></i></span>
            </a>
            {#if location.flightCount === 0}
              <a
                class="icon has-text-danger"
                title="Delete Location"
                href="/locations/{location.id}/delete"><i class="fa-solid fa-trash-alt"></i></a
              >
            {/if}
            {#if location.coordinates !== undefined}
              {@const lon = location.coordinates.lon}
              {@const lat = location.coordinates.lat}
              <a href="https://www.google.com/maps/place/{lat},{lon}/" title="View in Google Maps">
                <span class="icon"><i class="fa-solid fa-map-marker-alt"></i></span>
              </a>
              <a
                href="https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=16/{lat}/{lon}"
                title="View in OpenStreetMap"
              >
                <span class="icon"><i class="fa-solid fa-map-pin"></i></span>
              </a>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>
