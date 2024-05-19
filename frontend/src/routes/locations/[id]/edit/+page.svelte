<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import {i18n} from '$lib/i18n';

  import LocationForm from '../../LocationForm.svelte';

  import type {Data} from './+page';

  export let data: Data;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/locations/">{$i18n.t('navigation.locations')}</a></li>
    <li class="is-active"><a href="./" aria-current="page">{data.location.name}</a></li>
  </ul>
</nav>

<Flashes />

<LocationForm location={data.location}>
  <h2 slot="title" class="title is-2">
    {$i18n.t('location.title--edit-location', {name: data.location.name})}
  </h2>
</LocationForm>
