<script lang="ts">
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import XContestSummary from '$lib/components/XContestSummary.svelte';
  import {flightName} from '$lib/flights';
  import {formatDate, formatDistance, formatDuration, formatTime} from '$lib/formatters';
  import {i18n} from '$lib/i18n';

  import type {Data} from './+page';

  export let data: Data;

  $: flight = data.flight;
  $: launchAt = data.flight.launchAt;
  $: landingAt = data.flight.landingAt;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/flights/">{$i18n.t('navigation.flights')}</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">
        {$i18n.t('flight.title--flight')}
        {#if flight.number}#{flight.number}{:else}{flight.id}{/if}
      </a>
    </li>
  </ul>
</nav>

<Flashes />

<h2 class="title is-2">
  {flightName({...flight, launchAt: launchAt?.name, landingAt: landingAt?.name}, $i18n)}
</h2>

<div class="is-flex is-justify-content-space-between mb-5">
  <div class="left">
    <a href="/flights/{flight.id}/edit/" class="button is-light">
      {$i18n.t('flight.action--edit-this-flight')}
    </a>
  </div>
  <div class="right">
    {#if flight.hasIgc}
      <a href="/api/v1/flights/{flight.id}/igc/" class="button is-light" data-sveltekit-reload>
        <span class="icon is-small"><i class="fa-solid fa-download"></i></span>&nbsp;&nbsp;{$i18n.t(
          'flights.action--download-igc',
        )}
      </a>
    {/if}
  </div>
</div>

<section>
  <table class="table is-fullwidth is-striped is-hoverable">
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-list-ol"></i></span>&nbsp;&nbsp;{$i18n.t(
          'flight.title--flight-number',
        )}
      </th>
      <td>{flight.number || '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-parachute-box"></i></span>
        &nbsp;{$i18n.t('flight.title--glider')}
      </th>
      <td>{flight.gliderName || '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-calendar-alt"></i></span
        >&nbsp;&nbsp;{$i18n.t('flight.title--date')}
      </th>
      <td>{flight.launchTime ? formatDate(flight.launchTime) : '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-plane-departure"></i></span>
        &nbsp;{$i18n.t('flight.title--launch-site')}
      </th>
      <td>
        {#if launchAt}
          <CountryFlag countryCode={launchAt.countryCode} />
          <a href="/locations/{launchAt.id}/">{launchAt.name}</a>,
          {launchAt.elevation} mASL{:else}{$i18n.t(
            'flight.prose--unknown-site',
          )}{/if}{#if flight.launchTime}, {formatTime(flight.launchTime)} UTC{/if}
        {#if flight.hikeandfly}
          <i class="fa-solid fa-hiking" title="Hike &amp; Fly"></i>{/if}
      </td>
    </tr>
    <tr>
      <th
        ><span class="icon is-small"><i class="fa-solid fa-plane-arrival"></i></span
        >&nbsp;&nbsp;{$i18n.t('flight.title--landing-site')}</th
      >
      <td>
        {#if landingAt}
          <CountryFlag countryCode={landingAt.countryCode} />
          <a href="/locations/{landingAt.id}/">{landingAt.name}</a>,
          {landingAt.elevation} mASL{:else}{$i18n.t(
            'flight.prose--unknown-site',
          )}{/if}{#if flight.landingTime}, {formatTime(flight.landingTime)} UTC{/if}
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-clock"></i></span>
        &nbsp;{$i18n.t('flight.title--duration')}
      </th>
      <td>
        {flight.durationSeconds ? formatDuration(flight.durationSeconds) : '-'}
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-ruler"></i></span>&nbsp;&nbsp;{$i18n.t(
          'flight.title--gps-track-distance',
        )}
      </th>
      <td> {flight.trackDistance ? formatDistance(flight.trackDistance) : '-'}</td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-globe-americas"></i></span
        >&nbsp;&nbsp;{$i18n.t('flight.title--xcontest')}
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
        <span class="icon is-small"><i class="fa-solid fa-comment"></i></span>&nbsp;&nbsp;{$i18n.t(
          'flight.title--comment',
        )}
      </th>
      <td class="preserve-newlines">
        {flight.comment || '-'}
      </td>
    </tr>
    <tr>
      <th>
        <span class="icon is-small"><i class="fa-solid fa-film"></i></span>&nbsp;&nbsp;{$i18n.t(
          'flight.title--video-url',
        )}
      </th>
      <td>
        {#if flight.videoUrl}<a href={flight.videoUrl}>{flight.videoUrl}</a>{:else}-{/if}
      </td>
    </tr>
  </table>
</section>

<style>
  table th {
    white-space: nowrap;
  }
</style>
