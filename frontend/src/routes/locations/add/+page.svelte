<script lang="ts">
  import {onMount} from 'svelte';

  import {requireLogin} from '$lib/auth';
  import Flashes from '$lib/components/Flashes.svelte';
  import {i18n} from '$lib/i18n';
  import {loginState} from '$lib/stores';

  import LocationForm from '../LocationForm.svelte';

  onMount(() => {
    requireLogin($loginState, '/locations/add/');
  });
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/locations/">{$i18n.t('navigation.locations')}</a></li>
    <li class="is-active"><a href="./" aria-current="page">Add Location</a></li>
  </ul>
</nav>

<Flashes />

<LocationForm>
  <h2 slot="title" class="title is-2">Add Location</h2>
</LocationForm>
