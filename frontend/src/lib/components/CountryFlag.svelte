<!-- @component
Render a country flag using the FlagMeister SVG library.
-->
<script lang="ts">
  /**
   * The two-letter country code.
   */
  export let countryCode: string;
</script>

<!-- Custom elements from FlagMeister: E.g. <flag-ch></flag-ch> -->
<svelte:element this={`flag-${countryCode.toLowerCase()}`} class="flag"></svelte:element>

<style>
  .flag {
    display: inline-block;
  }

  .flag :global(img) {
    width: 1em;
    height: 0.8em;
  }
</style>
