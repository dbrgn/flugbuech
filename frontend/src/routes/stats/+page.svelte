<script lang="ts">
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import {formatDuration} from '$lib/formatters';

  import type {Data} from './+page';

  export let data: Data;

  $: sortedStats = Object.entries(data.yearlyStats)
    .map(([year, stats]) => ({year, ...stats}))
    .sort((a, b) => b.year.localeCompare(a.year));
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Stats</a></li>
  </ul>
</nav>

<Flashes />

<h2 class="title is-2">Stats</h2>

<section>
  <div class="columns">
    <div class="column is-half">
      <h3 class="title is-4">Yearly Stats</h3>

      <table class="table is-fullwidth is-striped is-hoverable is-narrow">
        <thead>
          <tr>
            <th>Year</th>
            <th>Flights</th>
            <th title="Hike &amp; Fly">H&amp;F</th>
            <th>Hours</th>
            <th>Track Distance</th>
            <th>Scored Distance</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedStats as stats}
            <tr>
              <td>{stats.year}</td>
              <td>{stats.flightCount}</td>
              <td>{stats.hikeandflyCount}</td>
              <td>
                {#if stats.flightSeconds}{formatDuration(stats.flightSeconds)} h{:else}?{/if}
              </td>
              <td>
                {stats.distance.track} km{#if stats.distanceTrackIncomplete}&nbsp;<sup
                    >{#if data.flightsWithoutLaunchTime > 0}2{:else}1{/if}</sup
                  >{/if}
              </td>
              <td
                >{stats.distance.scored} km{#if stats.distanceScoredIncomplete}&nbsp;<sup
                    >{#if data.flightsWithoutLaunchTime > 0}2{:else}1{/if}</sup
                  >{/if}</td
              >
            </tr>
          {/each}
          <tr class="has-text-weight-medium">
            <td>
              Total{#if data.flightsWithoutLaunchTime > 0}<sup>1</sup>{/if}
            </td>
            <td>{data.flightCountTotal}</td>
            <td>{data.hikeandflyCountTotal}</td>
            <td>{formatDuration(data.flightTimeTotal)}</td>
            <td>{data.flightDistanceTotal.track} km</td>
            <td>{data.flightDistanceTotal.scored} km</td>
          </tr>
        </tbody>
      </table>

      {#if data.flightsWithoutLaunchTime > 0}
        <p>
          <small>
            <sup>1</sup> Warning: There are {data.flightsWithoutLaunchTime} flights without launch date/time
            in your flight book, these will not contribute towards the yearly stats.
          </small>
        </p>
      {/if}

      <p>
        <small>
          <sup>{data.flightsWithoutLaunchTime > 0 ? 2 : 1}</sup> Data is incomplete (some flights don't
          contain this information)
        </small>
      </p>
    </div>

    <div class="column">
      <h3 class="title is-4">Top Launch Sites</h3>
      <table class="table is-fullwidth is-striped is-hoverable is-narrow">
        <thead>
          <tr>
            <th>Location</th>
            <th>Launches</th>
          </tr>
        </thead>
        <tbody>
          {#each data.launchLocations as location}
            <tr>
              <td>
                <CountryFlag countryCode={location.countryCode} />
                <a href="/locations/{location.id}">{location.name}</a>
              </td>
              <td>{location.flightCount}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="column">
      <h3 class="title is-4">Top Landing Sites</h3>
      <table class="table is-fullwidth is-striped is-hoverable is-narrow">
        <thead>
          <tr>
            <th>Location</th>
            <th>Landings</th>
          </tr>
        </thead>
        <tbody>
          {#each data.landingLocations as location}
            <tr>
              <td>
                <CountryFlag countryCode={location.countryCode} />
                <a href="/locations/{location.id}">{location.name}</a>
              </td>
              <td>{location.flightCount}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</section>
