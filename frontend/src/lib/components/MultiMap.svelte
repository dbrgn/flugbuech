<!-- Map showing a single location. May be editable. -->
<script lang="ts">
  import BaseMap from './BaseMap.svelte';
  import type {NamedCoordinates} from './map';

  export let markers: NamedCoordinates[];
</script>

<BaseMap mode="multi" {markers} />
