<script lang="ts">
  import {onMount} from 'svelte';

  import {requireLogin} from '$lib/auth';
  import Flashes from '$lib/components/Flashes.svelte';
  import {i18n} from '$lib/i18n';
  import {loginState} from '$lib/stores';

  import GliderForm from '../GliderForm.svelte';

  onMount(() => {
    requireLogin($loginState, '/flights/add/');
  });
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/gliders/">{$i18n.t('navigation.gliders')}</a></li>
    <li class="is-active">
      <a href="./" aria-current="page">{$i18n.t('glider.title--add-glider')}</a>
    </li>
  </ul>
</nav>

<Flashes />

<GliderForm>
  <h2 slot="title" class="title is-2">{$i18n.t('glider.title--add-glider')}</h2>
</GliderForm>
