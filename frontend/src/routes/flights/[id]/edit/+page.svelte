<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import {flightName} from '$lib/flights';
  import {i18n} from '$lib/i18n';

  import FlightForm from '../../FlightForm.svelte';

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

<FlightForm
  {flight}
  gliders={data.gliders}
  locations={data.locations}
  existingFlightNumbers={data.existingFlightNumbers}
>
  <h2 slot="title" class="title is-2">
    {$i18n.t('flight.title--edit-flight', {
      flight: flightName(
        {...data.flight, launchAt: launchAt?.name, landingAt: landingAt?.name},
        $i18n,
      ),
    })}
  </h2>
</FlightForm>
