<!-- @component
Render a XContest flight summary, with icon, distance and link.
-->
<script lang="ts">
  import {formatDistance} from '$lib/formatters';
  import {tracktypeName, type XContestTracktype} from '$lib/xcontest';

  import XContestTracktypeIcon from './XContestTracktypeIcon.svelte';

  /**
   * The track type.
   */
  export let tracktype: XContestTracktype | undefined;

  /**
   * The flight distance in km.
   */
  export let distance: number | undefined;

  /**
   * The XContest URL.
   */
  export let url: string | undefined;

  /**
   * Whether to format the link to XContest in subtle colors.
   */
  export let subtleLink = false;
</script>

{#if tracktype === undefined && distance === undefined && url === undefined}
  -
{:else if url}
  <a class:subtle-link={subtleLink} href={url}>
    {#if tracktype}
      <XContestTracktypeIcon {tracktype} />
    {/if}
    {#if distance}
      {formatDistance(distance)}
    {:else if tracktype}
      {tracktypeName(tracktype)}
    {:else}
      Flight
    {/if}
  </a>
{:else}
  {#if tracktype}
    <XContestTracktypeIcon {tracktype} />
  {/if}
  {#if distance}
    {formatDistance(distance)}
  {:else if tracktype}
    {tracktypeName(tracktype)}
  {:else}
    Flight
  {/if}
{/if}
