<script lang="ts">
  import {onMount} from 'svelte';

  import Flashes from '$lib/components/Flashes.svelte';
  import SubstitutableText from '$lib/components/SubstitutableText.svelte';
  import {i18n} from '$lib/i18n';
  import {ResolvablePromise} from '$lib/resolvable-promise';

  import {_loadApiStats, type GlobalStats} from './+page';

  let stats = new ResolvablePromise<GlobalStats>();

  onMount(() => {
    // Load stats from API
    _loadApiStats().then((result) => stats.resolve(result));
  });
</script>

<Flashes />

<article class="message is-primary">
  <div class="message-header">
    <p>{$i18n.t('home.title--update-march-2024')}</p>
  </div>
  <div class="message-body">
    <SubstitutableText text={$i18n.t('home.news.2024-03-13')}>
      <a slot="1" href="mailto:flugbuech@bargen.dev" let:text>{text}</a>
    </SubstitutableText>
  </div>
</article>

<h2 class="title is-size-2">{$i18n.t('home.title--overview')}</h2>

<p class="content">
  <SubstitutableText text={$i18n.t('home.prose--intro')}>
    <em slot="1" let:text>{text}</em>
  </SubstitutableText>
</p>

<div class="content">
  <SubstitutableText text={$i18n.t('home.prose--what-is')}>
    <em slot="1" let:text>{text}</em>
  </SubstitutableText>
  <ul>
    <li>
      <SubstitutableText text={$i18n.t('home.prose--what-is-summary')}>
        <strong slot="1" let:text>{text}</strong>
      </SubstitutableText>
    </li>
    <li>
      <SubstitutableText text={$i18n.t('home.prose--what-is-goal')}>
        <strong slot="1" let:text>{text}</strong>
      </SubstitutableText>
    </li>
    <li>
      <SubstitutableText text={$i18n.t('home.prose--what-is-additional-information')}>
        <strong slot="1" let:text>{text}</strong>
      </SubstitutableText>
    </li>
    <li>
      <SubstitutableText text={$i18n.t('home.prose--what-is-stats')}>
        <strong slot="1" let:text>{text}</strong>
      </SubstitutableText>
    </li>
    <li>
      <SubstitutableText text={$i18n.t('home.prose--what-is-no-ads')}>
        <strong slot="1" let:text>{text}</strong>
      </SubstitutableText>
    </li>
  </ul>
</div>

<p class="content">
  <SubstitutableText text={$i18n.t('home.prose--screenshots')}>
    <a slot="1" href="/screenshots/" let:text>{text}</a>
    <a slot="2" href="https://github.com/dbrgn/flugbuech#status" let:text>{text}</a>
  </SubstitutableText>
</p>

<h3 class="title is-size-3">{$i18n.t('home.title--faq')}</h3>

<h4 class="title is-size-5">{$i18n.t('home.faq.is-it-free')}</h4>
<p class="content">
  {$i18n.t('home.faq.is-it-free-answer-1')}
</p>

<h4 class="title is-size-5">{$i18n.t('home.faq.signup')}</h4>
<p class="content">
  <SubstitutableText text={$i18n.t('home.faq.signup--answer-1')}>
    <a slot="1" href="/auth/registration/" let:text>{text}</a>
  </SubstitutableText>
</p>
<p class="content">
  <SubstitutableText text={$i18n.t('home.faq.signup--answer-2')}>
    <a slot="1" href="mailto:flugbuech@bargen.dev" let:text>{text}</a>
  </SubstitutableText>
</p>

<h4 class="title is-size-5">{$i18n.t('home.faq.import')}</h4>
<p class="content">
  <SubstitutableText text={$i18n.t('home.faq.import--answer-1')}>
    <a slot="1" href="/flights/import/csv/" let:text>{text}</a>
  </SubstitutableText>
</p>

<h4 class="title is-size-5">
  {$i18n.t('home.faq.source')}
</h4>
<p class="content">
  <SubstitutableText text={$i18n.t('home.faq.source-answer-1')}>
    <a slot="1" href="https://github.com/dbrgn/flugbuech" let:text>{text}</a>
    <a slot="2" href="mailto:flugbuech@bargen.dev" let:text>{text}</a>
  </SubstitutableText>
</p>

<h4 class="title is-size-5">{$i18n.t('home.faq.name')}</h4>
<p class="content">
  <SubstitutableText text={$i18n.t('home.faq.name-answer-1')}>
    <a slot="1" href="https://en.wikipedia.org/wiki/Swiss_German" let:text>{text}</a>
  </SubstitutableText>
</p>

<h3 class="title is-size-3">{$i18n.t('home.title--news')}</h3>
<h4 class="title is-size-5">2024</h4>
<ul class="content">
  <li>
    <strong>25.5.</strong>
    {$i18n.t('home.news.2024-05-25')}
  </li>
  <li>
    <strong>14.5.</strong>
    <SubstitutableText text={$i18n.t('home.news.2024-05-14')}>
      <a slot="1" href="/locations/" let:text>{text}</a>
    </SubstitutableText>
  </li>
  <li>
    <strong>1.4.</strong>
    <SubstitutableText text={$i18n.t('home.news.2024-04-01')}>
      <a slot="1" href="/flights/import/csv/" let:text>{text}</a>
    </SubstitutableText>
  </li>
  <li>
    <strong>19.3.</strong>
    <SubstitutableText text={$i18n.t('home.news.2024-03-19')}>
      <a slot="1" href="/profile/" let:text>{text}</a>
    </SubstitutableText>
  </li>
  <li>
    <strong>13.3.</strong>
    <SubstitutableText text={$i18n.t('home.news.2024-03-13')}>
      <a slot="1" href="mailto:flugbuech@bargen.dev" let:text>{text}</a>
    </SubstitutableText>
  </li>
</ul>
<h4 class="title is-size-5">2019­–2023</h4>
<ul class="content">
  <li>
    <strong>2023</strong>
    {$i18n.t('home.news.2023')}
  </li>
  <li>
    <strong>2021</strong>
    {$i18n.t('home.news.2021')}
  </li>
  <li>
    <strong>2020</strong>
    {$i18n.t('home.news.2020')}
  </li>
  <li>
    <strong>2019</strong>
    {$i18n.t('home.news.2019')}
  </li>
</ul>

<h3 class="title is-size-3">Stats</h3>
<table class="table is-bordered is-hoverable">
  <tbody>
    <tr>
      <th>Registered Users</th>
      <td>
        {#await stats}
          Loading...
        {:then result}
          {result.userCount}
        {/await}
      </td>
    </tr>
    <tr>
      <th>Registered Gliders</th>
      <td>
        {#await stats}
          Loading...
        {:then result}
          {result.gliderCount}
        {/await}
      </td>
    </tr>
    <tr>
      <th>Total Flights</th>
      <td>
        {#await stats}
          Loading...
        {:then result}
          {result.flightCount}
        {/await}
      </td>
    </tr>
  </tbody>
</table>
