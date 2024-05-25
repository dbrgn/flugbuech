<script lang="ts">
  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
  import {formatDuration} from '$lib/formatters';
  import {i18n} from '$lib/i18n';

  import type {Data} from './+page';

  export let data: Data;

  $: sortedStats = Object.entries(data.yearlyStats)
    .map(([year, stats]) => ({year, ...stats}))
    .sort((a, b) => b.year.localeCompare(a.year));
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li class="is-active"><a href="./" aria-current="page">{$i18n.t('navigation.stats')}</a></li>
  </ul>
</nav>

<Flashes />

<h2 class="title is-2">{$i18n.t('stats.title--stats')}</h2>

<section>
  <div class="columns">
    <div class="column is-half">
      <h3 class="title is-4">{$i18n.t('stats.title--yearly-stats')}</h3>

      <table class="table is-fullwidth is-striped is-hoverable is-narrow">
        <thead>
          <tr>
            <th>{$i18n.t('stats.column--year')}</th>
            <th>{$i18n.t('stats.column--flights')}</th>
            <th title="Hike &amp; Fly">H&amp;F</th>
            <th>{$i18n.t('stats.column--hours')}</th>
            <th>{$i18n.t('stats.column--track-distance')}</th>
            <th>{$i18n.t('stats.column--scored-distance')}</th>
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
              {$i18n.t('stats.column--total', 'Total')}{#if data.flightsWithoutLaunchTime > 0}<sup
                  >1</sup
                >{/if}
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
            <sup>1</sup>
            {$i18n.t('stats.prose--warning-flights-without-time', {
              count: data.flightsWithoutLaunchTime,
            })}
          </small>
        </p>
      {/if}

      <p>
        <small>
          <sup>{data.flightsWithoutLaunchTime > 0 ? 2 : 1}</sup>
          {$i18n.t('stats.prose--warning-data-incomplete')}
        </small>
      </p>
    </div>

    <div class="column">
      <h3 class="title is-4">{$i18n.t('stats.title--top-launch-sites')}</h3>
      <table class="table is-fullwidth is-striped is-hoverable is-narrow">
        <thead>
          <tr>
            <th>{$i18n.t('stats.column--location')}</th>
            <th>{$i18n.t('stats.column--launches')}</th>
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
      <h3 class="title is-4">{$i18n.t('stats.title--top-landing-sites')}</h3>
      <table class="table is-fullwidth is-striped is-hoverable is-narrow">
        <thead>
          <tr>
            <th>{$i18n.t('stats.column--location')}</th>
            <th>{$i18n.t('stats.column--landings')}</th>
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
