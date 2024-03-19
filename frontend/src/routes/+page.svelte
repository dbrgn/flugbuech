<script lang="ts">
  import {onMount} from 'svelte';

  import Flashes from '$lib/components/Flashes.svelte';
  import {ResolvablePromise} from '$lib/resolvable-promise';

  import {_loadApiStats, type GlobalStats} from './+page';

  let stats = new ResolvablePromise<GlobalStats>();

  onMount(() => {
    // Load stats from API
    _loadApiStats().then((result) => stats.resolve(result));
  });
</script>

<Flashes />

<article class="message is-info">
  <div class="message-header">
    <p>Update March 2024</p>
  </div>
  <div class="message-body">
    After a longer development break, the complete Flugbuech UI has been rewritten from scratch! It
    should still look (mostly) the same, but it should react faster for you and be easier to
    maintain and extend for me. If you have any feedback (positive or negative), contact me at
    <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a> (German or English)!
  </div>
</article>

<h2 class="title is-size-2">Overview</h2>

<p class="content">
  Welcome to Flugbuech! This is a free, open source and ad-free platform for keeping track of your
  free flights (paragliding or hang gliding).
</p>

<p class="content">
  The software is mostly aimed at pilots that track their flights using a flight computer which
  generates IGC files, but it can also be used without an IGC file for every flight. My goal is to
  extract as much flight data from the IGC file as possible (e.g. launch site, landing site,
  distance, etc), so that you don't need to enter that data manually. The IGC files are stored
  together with the flight data.
</p>

<p class="content">
  You can find the list of features and the current project status <a
    href="https://github.com/dbrgn/flugbuech#status">on the project GitHub page</a
  >.
</p>

<h3 class="title is-size-3">FAQ</h3>

<h4 class="title is-size-5">Is it free?</h4>
<p class="content">
  Yes, and there are no ads and no user tracking! Usually free things have a catch, but in this case
  I'm developing the flight log for my own needs, and others may use it too.
</p>

<h4 class="title is-size-5">Can I sign up?</h4>
<p class="content">
  Sure thing! Feel free to <a href="/auth/registration"> create an account</a>.
</p>
<p class="content">
  Keep in mind that this is still beta software and that there may be bugs. If you notice any
  problems or have ideas for new features, please let me know at
  <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a>.
</p>

<h4 class="title is-size-5">Where can I find the source code or report bugs?</h4>
<p class="content">On <a href="https://github.com/dbrgn/flugbuech">GitHub</a>.</p>

<h4 class="title is-size-5">What does &laquo;Flugbuech&raquo; mean?</h4>
<p class="content">
  It's <a href="https://en.wikipedia.org/wiki/Swiss_German">Swiss German</a> for &laquo;Flight Log&raquo;.
  Sorry for not being more creative.
</p>

<h3 class="title is-size-3">News</h3>
<h4 class="title is-size-5">2024</h4>
<ul class="content">
  <li>
    <strong>19.3.</strong> You can now opt-in to an occasional newsletter
    <a href="/profile/">in your user profile</a>!
  </li>
  <li>
    <strong>13.03.</strong> After a longer development break, the complete Flugbuech UI has been
    rewritten from scratch! It should still look (mostly) the same, but it should react faster for
    you and be easier to maintain and extend for me. If you have any feedback (positive or
    negative), contact me at <a href="mailto:flugbuech@bargen.dev">flugbuech@bargen.dev</a> (German or
    English)!
  </li>
</ul>
<h4 class="title is-size-5">2023</h4>
<ul class="content">
  <li>
    <strong>21.12.</strong> There is now a <a href="/privacy-policy/">privacy policy</a>
  </li>
</ul>
<h4 class="title is-size-5">2021</h4>
<ul class="content">
  <li><strong>20.09.</strong> User registration is implemented</li>
  <li>
    <strong>20.04.</strong> Locations can now be viewed and deleted, number of associated flights is
    listed
  </li>
  <li>
    <strong>19.04.</strong> Location editing now supports
    <a href="https://www.swisstopo.admin.ch/de/karten-daten-online/karten-geodaten-online.html"
      >Swisstopo</a
    > maps
  </li>
</ul>
<h4 class="title is-size-5">2020</h4>
<ul class="content">
  <li><strong>25.09.</strong> Show stats (flights, flight hours) for gliders</li>
  <li>
    <strong>11.07.</strong> Allow marking flights as hike&amp;fly, allow deleting flights, enhance stats
  </li>
  <li>
    <strong>10.07.</strong> Your own password
    <a href="/auth/password/change/">can now be changed</a>
  </li>
  <li><strong>14.06.</strong> Allow editing your gliders</li>
  <li>
    <strong>12.06.</strong> Statistics about your flights have been added! Check out
    <a href="/stats/">/stats/</a>.
  </li>
</ul>
<h4 class="title is-size-5">2019</h4>
<ul class="content">
  <li><strong>18.12.</strong> IGC files from XContest can now be parsed properly</li>
</ul>

<h3 class="title is-size-3">Stats</h3>
<table class="table is-bordered is-hoverable">
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
</table>
