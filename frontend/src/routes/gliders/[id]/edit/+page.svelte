<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';

  import GliderForm from '../../GliderForm.svelte';
  import type {Data} from './+page';

  export let data: Data;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/gliders/">Gliders</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">{data.glider.manufacturer} {data.glider.model}</a>
    </li>
  </ul>
</nav>

<Flashes />

<GliderForm glider={data.glider}>
  <h2 slot="title" class="title is-2">
    Edit Glider: {data.glider.manufacturer}
    {data.glider.model}
  </h2>
</GliderForm>
