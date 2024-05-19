<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import {i18n} from '$lib/i18n';

  import GliderForm from '../../GliderForm.svelte';

  import type {Data} from './+page';

  export let data: Data;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/gliders/">{$i18n.t('navigation.gliders')}</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">{data.glider.manufacturer} {data.glider.model}</a>
    </li>
  </ul>
</nav>

<Flashes />

<GliderForm glider={data.glider}>
  <h2 slot="title" class="title is-2">
    {$i18n.t('glider.title--edit-glider', {
      manufacturer: data.glider.manufacturer,
      model: data.glider.model,
    })}
  </h2>
</GliderForm>
