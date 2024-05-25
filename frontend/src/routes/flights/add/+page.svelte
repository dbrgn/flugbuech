<script lang="ts">
  import {onMount} from 'svelte';

  import {requireLogin} from '$lib/auth';
  import Flashes from '$lib/components/Flashes.svelte';
  import SubstitutableText from '$lib/components/SubstitutableText.svelte';
  import {i18n} from '$lib/i18n';
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
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/flights/">{$i18n.t('navigation.flights')}</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">{$i18n.t('flights.action--add-flight')}</a>
    </li>
  </ul>
</nav>

<Flashes />

<FlightForm
  gliders={data.gliders}
  lastGliderId={data.lastGliderId}
  locations={data.locations}
  existingFlightNumbers={data.existingFlightNumbers}
>
  <h2 slot="title" class="title is-2">{$i18n.t('flight.title--add-flight')}</h2>

  <section slot="intro">
    <p class="content">
      {$i18n.t('flight.prose--fill-out-form')}
    </p>

    <p class="content">
      <SubstitutableText text={$i18n.t('flight.prose--note-add-location')}>
        <strong slot="1" let:text>{text}</strong>
        <a slot="2" href="/locations/add/" let:text>{text}</a>
      </SubstitutableText>
    </p>

    {#if data.locations.length === 0}
      <article class="message is-warning">
        <div class="message-body">
          <i class="fa-solid fa-warning" />&ensp;<SubstitutableText
            text={$i18n.t('flight.prose--note-missing-location')}
          >
            <strong slot="1" let:text>{text}</strong>
            <a slot="2" href="/locations/add/" let:text>{text}</a>
          </SubstitutableText>
        </div>
      </article>
    {/if}

    {#if data.gliders.length === 0}
      <article class="message is-warning">
        <div class="message-body">
          <i class="fa-solid fa-warning" />&ensp;<SubstitutableText
            text={$i18n.t('flight.prose--note-missing-gliders')}
          >
            <strong slot="1" let:text>{text}</strong>
            <a slot="2" href="/gliders/" let:text>{text}</a>
          </SubstitutableText>
        </div>
      </article>
    {/if}
  </section>
</FlightForm>
