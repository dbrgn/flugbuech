<script>
  import {i18n} from '$lib/i18n';
</script>

<h2 class="title is-size-2">Screenshots</h2>

<p>{$i18n.t('screenshots.language-hint', 'Aktuell nur auf Englisch verfügbar.')}</p>

<div class="content item">
  <h3 class="title">Flight List</h3>
  <p>The flight list shows all your flights in a compact table.</p>
  <img src="/screenshots/flight-list.png" alt="Screenshot of the flight list" />
</div>

<div class="content item">
  <h3 class="title">Flight Details</h3>
  <p>You can view more details about your flight in the flight details view.</p>
  <img src="/screenshots/flight-details.png" alt="Screenshot of the flight details" />
</div>

<div class="content item">
  <h3 class="title">Location List</h3>
  <p>You can manage your launch and landing locations yourself.</p>
  <img src="/screenshots/location-list.png" alt="Screenshot of the location list" />
</div>

<div class="content item">
  <h3 class="title">Location Editing</h3>
  <p>
    The location can be edited using a map (both Mapbox and Swisstopo are available as map options).
  </p>
  <img src="/screenshots/location-edit.png" alt="Screenshot of the location editing" />
</div>

<div class="content item">
  <h3 class="title">Stats</h3>
  <p>You can view various statistics about your flights.</p>
  <img src="/screenshots/stats.png" alt="Screenshot of the stats" />
</div>

<style>
  .item {
    margin-top: 4em;
  }

  .item img {
    border: 1px solid #ddd;
  }
</style>
