<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';

  import LocationForm from '../../LocationForm.svelte';
  import type {Data} from './+page';

  export let data: Data;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/locations">Locations</a></li>
    <li class="is-active"><a href="./" aria-current="page">{data.location.name}</a></li>
  </ul>
</nav>

<Flashes />

<LocationForm location={data.location}>
  <h2 slot="title" class="title is-2">Edit Location: {data.location.name}</h2>
</LocationForm>
