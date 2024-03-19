<script lang="ts">
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import MapComponent from '$lib/components/Map.svelte';

  import type {Data} from './+page';

  export let data: Data;

  $: location = data.location;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/locations/">Locations</a></li>
    <li class="is-active"><a href="./" aria-current="page">{location.name}</a></li>
  </ul>
</nav>

<Flashes />

<h2 class="title is-2">Location: {location.name}</h2>

<p class="content">
  <a href="/locations/{location.id}/edit/" class="button is-light">Edit this location</a>
</p>

<section class="data">
  <table class="table is-fullwidth is-striped is-hoverable">
    <tbody>
      <tr>
        <th><span class="icon is-small"><i class="fas fa-map-signs"></i></span>&nbsp;&nbsp;Name</th>
        <td>{location.name}</td>
      </tr>
      <tr>
        <th><span class="icon is-small"><i class="fas fa-globe"></i></span>&nbsp;&nbsp;Country</th>
        <td><CountryFlag countryCode={location.countryCode} /> {location.countryCode}</td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-arrow-up"></i></span>&nbsp;&nbsp;Elevation
        </th>
        <td>{location.elevation} m ASL</td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-map-marked-alt"></i></span
          >&nbsp;&nbsp;Coordinates (EPSG 3857)
        </th>
        <td>
          {#if location.coordinates}{location.coordinates.lat}, {location.coordinates.lon}{/if}
        </td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-parachute-box"></i></span
          >&nbsp;&nbsp;Associated Flights
        </th>
        <td>{location.flightCount}</td>
      </tr>
    </tbody>
  </table>
</section>

{#if location.coordinates}
  <section class="map">
    <MapComponent editable={false} position={location.coordinates} zoom={13} />
  </section>
{/if}

<style>
  .map {
    margin-top: 32px;
  }
</style>
