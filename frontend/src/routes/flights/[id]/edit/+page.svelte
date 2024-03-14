<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';

  import FlightForm from '../../FlightForm.svelte';

  import type {Data} from './+page';

  export let data: Data;

  $: flight = data.flight;
  $: launchAt = data.flight.launchAt;
  $: landingAt = data.flight.landingAt;
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

<FlightForm
  {flight}
  gliders={data.gliders}
  locations={data.locations}
  existingFlightNumbers={data.existingFlightNumbers}
>
  <h2 slot="title" class="title is-2">
    Edit Flight
    {#if flight.number}#{flight.number}{/if}
    {#if launchAt}from {launchAt.name}{/if}
    {#if landingAt}to {landingAt.name}{/if}
  </h2>
</FlightForm>
