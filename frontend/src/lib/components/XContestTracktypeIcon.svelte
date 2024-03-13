<!-- @component
Render the XContest track type.
-->
<script lang="ts">
  import type {XContestTracktype} from '$lib/xcontest';

  /**
   * The track type.
   */
  export let tracktype: XContestTracktype;
</script>

<img
  src="/img/xcontest_{tracktype}.gif"
  alt={tracktype}
  title={tracktype}
  class="xcontest-tracktype"
/>

<style>
  .xcontest-tracktype {
    margin-bottom: -3px;
  }
</style>
