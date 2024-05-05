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

<div class="is-flex is-justify-content-space-between mb-5">
  <div class="left">
    <a href="/locations/{location.id}/edit/" class="button is-light">Edit this location</a>
  </div>
  <div class="right">
    {#if location.coordinates !== undefined}
      {@const lon = location.coordinates.lon}
      {@const lat = location.coordinates.lat}
      <a
        class="button is-light"
        href="https://www.google.com/maps/place/{lat},{lon}/"
        title="View on Google Maps"
        target="_blank"
      >
        <i class="fa-solid fa-map-marker-alt"></i>&nbsp;View on Google Maps
      </a>
      <a
        class="button is-light"
        href="https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=16/{lat}/{lon}"
        title="View on OpenStreetMap"
        target="_blank"
      >
        <i class="fa-solid fa-map-pin"></i>&nbsp;View on OpenStreetMap
      </a>
      {#if lat > 45.8 && lat < 48 && lon > 5.9 && lon < 11.4}
        <a
          class="button is-light"
          href="https://map.geo.admin.ch/?swisssearch={lat},{lon}"
          title="View on SwissTopo"
          target="_blank"
        >
          <i class="fa-solid fa-map"></i>&nbsp;View on SwissTopo
        </a>
      {/if}
    {/if}
  </div>
</div>

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
