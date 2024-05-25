<script lang="ts">
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import SingleMap from '$lib/components/SingleMap.svelte';
  import {i18n} from '$lib/i18n';

  import type {Data} from './+page';

  export let data: Data;

  $: location = data.location;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/locations/">{$i18n.t('navigation.locations')}</a></li>
    <li class="is-active"><a href="./" aria-current="page">{location.name}</a></li>
  </ul>
</nav>

<Flashes />

<h2 class="title is-2">
  {$i18n.t('location.title--location', {name: location.name})}
</h2>

<div class="is-flex is-justify-content-space-between mb-5">
  <div class="left">
    <a href="/locations/{location.id}/edit/" class="button is-light">
      {$i18n.t('location.action--edit')}
    </a>
  </div>
  <div class="right">
    {#if location.coordinates !== undefined}
      {@const lon = location.coordinates.lon}
      {@const lat = location.coordinates.lat}
      <a
        class="button is-light"
        href="https://www.google.com/maps/place/{lat},{lon}/"
        target="_blank"
      >
        <i class="fa-solid fa-map-marker-alt"></i>&nbsp;
        {$i18n.t('locations.action--view-google-maps')}
      </a>
      <a
        class="button is-light"
        href="https://www.openstreetmap.org/?mlat={lat}&mlon={lon}#map=16/{lat}/{lon}"
        target="_blank"
      >
        <i class="fa-solid fa-map-pin"></i>&nbsp;
        {$i18n.t('locations.action--view-osm')}
      </a>
      {#if lat > 45.8 && lat < 48 && lon > 5.9 && lon < 11.4}
        <a
          class="button is-light"
          href="https://map.geo.admin.ch/?swisssearch={lat},{lon}"
          target="_blank"
        >
          <i class="fa-solid fa-map"></i>&nbsp
          {$i18n.t('locations.action--view-swisstopo')}
        </a>
      {/if}
    {/if}
  </div>
</div>

<section class="data">
  <table class="table is-fullwidth is-striped is-hoverable">
    <tbody>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-map-signs"></i></span>&nbsp;&nbsp;{$i18n.t(
            'location.title--name',
          )}
        </th>
        <td>{location.name}</td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-globe"></i></span>&nbsp;&nbsp;{$i18n.t(
            'location.title--country',
          )}
        </th>
        <td><CountryFlag countryCode={location.countryCode} /> {location.countryCode}</td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-arrow-up"></i></span>&nbsp;&nbsp;{$i18n.t(
            'location.title--elevation',
          )}
        </th>
        <td>{location.elevation} {$i18n.t('common.unit--m-asl')}</td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-map-marked-alt"></i></span
          >&nbsp;&nbsp;{$i18n.t('location.title--coordinates')} (WGS 84)
        </th>
        <td>
          {#if location.coordinates}{location.coordinates.lat}, {location.coordinates.lon}{/if}
        </td>
      </tr>
      <tr>
        <th>
          <span class="icon is-small"><i class="fas fa-parachute-box"></i></span
          >&nbsp;&nbsp;{$i18n.t('location.title--associated-flights')}
        </th>
        <td>{location.flightCount}</td>
      </tr>
    </tbody>
  </table>
</section>

{#if location.coordinates}
  <section class="map">
    <SingleMap editable={false} center={location.coordinates} zoom={13} />
  </section>
{/if}

<style>
  .map {
    margin-top: 32px;
  }
</style>
