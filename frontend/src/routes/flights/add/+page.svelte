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
  </section>
</FlightForm>
