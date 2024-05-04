<script lang="ts">
  import {unreachable} from '$lib/assert';

  export let href: string;
  export let text: string;
  export let type: 'link' | 'button' = 'link';
  export let reload = false;
  export let closeMenu: () => void;

  let linkClass: string;
  $: switch (type) {
    case 'link':
      linkClass = 'navbar-item';
      break;
    case 'button':
      linkClass = 'button';
      break;
    default:
      unreachable(type);
  }
</script>

<a class={linkClass} {href} on:click={() => closeMenu()} data-sveltekit-reload={reload || null}>
  {text}
</a>
