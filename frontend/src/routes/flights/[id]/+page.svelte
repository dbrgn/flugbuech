<script lang="ts">
  import {bytesToHex} from '$lib/byte';
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import XContestSummary from '$lib/components/XContestSummary.svelte';
  import {randomBytes} from '$lib/crypto';
  import {formatDate, formatDistance, formatDuration, formatTime} from '$lib/formatters';

  import type {Data} from './+page';

  export let data: Data;

  $: flight = data.flight;
  $: launchAt = data.flight.launchAt;
  $: landingAt = data.flight.landingAt;

  const xcontestX = bytesToHex(randomBytes(new Uint8Array(16)));
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/flights/">Flights</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">
        Flight {#if flight.number}#{flight.number}{:else}{flight.id}{/if}
      </a>
    </li>
  </ul>
</nav>

<Flashes />

<h2 class="title is-2">
  Flight
  {#if flight.number}#{flight.number}{/if}
  {#if launchAt}from {launchAt.name}{/if}
  {#if landingAt}to {landingAt.name}{/if}
</h2>

<p class="content">
  <a href="/flights/{flight.id}/edit/" class="button is-light">Edit this flight</a>
  {#if flight.hasIgc}
    <a href="/api/v1/flights/{flight.id}/igc/" class="button is-light" data-sveltekit-reload>
      <span class="icon is-small"><i class="fa-solid fa-download"></i></span>&nbsp;&nbsp;Download
      IGC
    </a>
  {/if}
</p>

<section>
  <table class="table is-fullwidth is-striped is-hoverable">
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-list-ol"></i></span>&nbsp;&nbsp;Number
      </th>
      <td>{flight.number || '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-parachute-box"></i></span>
        &nbsp;Glider
      </th>
      <td>{flight.gliderName || '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-calendar-alt"></i></span>&nbsp;&nbsp;Date
      </th>
      <td>{flight.launchTime ? formatDate(flight.launchTime) : '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-plane-departure"></i></span>
        &nbsp;Launch
      </th>
      <td>
        {#if launchAt}
          <CountryFlag countryCode={launchAt.countryCode} />
          <a href="/locations/{launchAt.id}/">{launchAt.name}</a>,
          {launchAt.elevation} mASL{:else}Unknown site{/if}{#if flight.launchTime}, {formatTime(
            flight.launchTime,
          )} UTC{/if}
        {#if flight.hikeandfly}
          <i class="fa-solid fa-hiking" title="Hike &amp; Fly"></i>{/if}
      </td>
    </tr>
    <tr>
      <th
        ><span class="icon is-small"><i class="fa-solid fa-plane-arrival"></i></span
        >&nbsp;&nbsp;Landing</th
      >
      <td>
        {#if landingAt}
          <CountryFlag countryCode={landingAt.countryCode} />
          <a href="/locations/{landingAt.id}/">{landingAt.name}</a>,
          {landingAt.elevation} mASL{:else}Unknown site{/if}{#if flight.landingTime}, {formatTime(
            flight.landingTime,
          )} UTC{/if}
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-clock"></i></span>
        &nbsp;Duration
      </th>
      <td>
        {flight.durationSeconds ? formatDuration(flight.durationSeconds) : '-'}
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-ruler"></i></span>&nbsp;&nbsp;GPS Track
        Distance
      </th>
      <td> {flight.trackDistance ? formatDistance(flight.trackDistance) : '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-globe-americas"></i></span
        >&nbsp;&nbsp;XContest
      </th>
      <td>
        <XContestSummary
          tracktype={flight.xcontestTracktype}
          distance={flight.xcontestDistance}
          url={flight.xcontestUrl}
        />
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-comment"></i></span>&nbsp;&nbsp;Comment
      </th>
      <td class="preserve-newlines">
        {flight.comment || '-'}
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-film"></i></span>&nbsp;&nbsp;Video URL
      </th>
      <td>
        {#if flight.videoUrl}<a href={flight.videoUrl}>{flight.videoUrl}</a>{:else}-{/if}
      </td>
    </tr>
  </table>
</section>

<form
  method="post"
  action="https://www.xcontest.org/world/en/flight-claim/"
  enctype="multipart/form-data"
  rel="external noreferrer"
  target="_blank"
>
  <input type="file" name="flight[tracklog]" />
  <input type="text" name="flight[__x__]" value={xcontestX} />
  <button class="button" type="submit">Go</button>
</form>

<style>
  table th {
    white-space: nowrap;
  }
</style>
