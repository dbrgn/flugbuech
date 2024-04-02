<script lang="ts">
  import {onMount} from 'svelte';

  import {requireLogin} from '$lib/auth';
  import Flashes from '$lib/components/Flashes.svelte';
  import {loginState} from '$lib/stores';

  import FlightForm from '../FlightForm.svelte';

  import type {Data} from './+page';

  export let data: Data;

  onMount(() => {
    requireLogin($loginState, '/flights/add/');
  });
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/flights/">Flights</a></li>
    <li class="is-active"><a href="./" aria-current="page">Add Flight</a></li>
  </ul>
</nav>

<Flashes />

<FlightForm
  gliders={data.gliders}
  lastGliderId={data.lastGliderId}
  locations={data.locations}
  existingFlightNumbers={data.existingFlightNumbers}
>
  <h2 slot="title" class="title is-2">Add Flight</h2>

  <section slot="intro">
    <p class="content">
      Fill out this form to submit a flight. If you upload an IGC file, some of the fields will be
      automatically filled for you.
    </p>

    <p class="content">
      <strong>NOTE:</strong> If the launch or landing location is still missing, please
      <a href="/locations/add">add it</a> before filling out this form! Currently locations cannot be
      created automatically.
    </p>

    {#if data.locations.length === 0}
      <article class="message is-warning">
        <div class="message-body">
          <i class="fa-solid fa-warning" />&ensp;<strong>Warning:</strong> You haven't added any
          locations so far. To be able to link a launch or landing location to your flight,
          <a href="/locations/">add a location</a> before adding a flight.
        </div>
      </article>
    {/if}

    {#if data.gliders.length === 0}
      <article class="message is-warning">
        <div class="message-body">
          <i class="fa-solid fa-warning" />&ensp;<strong>Warning:</strong> You haven't added any
          gliders so far. To be able to link a glider to your flight,
          <a href="/gliders/">add a glider</a> before adding a flight.
        </div>
      </article>
    {/if}
  </section>
</FlightForm>
