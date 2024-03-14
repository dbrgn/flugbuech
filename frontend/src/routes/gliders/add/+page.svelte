<script lang="ts">
  import {onMount} from 'svelte';

  import {requireLogin} from '$lib/auth';
  import Flashes from '$lib/components/Flashes.svelte';
  import {loginState} from '$lib/stores';

  import GliderForm from '../GliderForm.svelte';

  onMount(() => {
    requireLogin($loginState, '/flights/add/');
  });
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li><a href="/gliders/">Gliders</a></li>
    <li class="is-active"><a href="./" aria-current="page">Add Glider</a></li>
  </ul>
</nav>

<Flashes />

<GliderForm>
  <h2 slot="title" class="title is-2">Add Glider</h2>
</GliderForm>
