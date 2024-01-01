<script lang="ts">
  import LocationForm from '../LocationForm.svelte';
  import Flashes from '$lib/components/Flashes.svelte';
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/locations">Locations</a></li>
    <li class="is-active"><a href="./" aria-current="page">Add Location</a></li>
  </ul>
</nav>

<Flashes />

<LocationForm>
  <h2 slot="title" class="title is-2">Add Location</h2>
</LocationForm>
