<!-- Map showing a single location. May be editable. -->
<script lang="ts">
  import type {LngLatLike} from 'maplibre-gl';

  import BaseMap from './BaseMap.svelte';
  import {DEFAULT_MAP_CENTER} from './map';

  export let latitude: number | null = null;
  export let longitude: number | null = null;
  export let elevation: number | null = null;
  export let editable: boolean = false;
  export let center: LngLatLike = DEFAULT_MAP_CENTER;
  export let zoom: number = 6;
</script>

<BaseMap mode="single" bind:latitude bind:longitude bind:elevation {editable} {center} {zoom} />
