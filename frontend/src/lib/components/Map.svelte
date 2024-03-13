<script lang="ts">
  import {reactive} from '$lib/svelte';
  import {Map, NavigationControl, Marker, type LngLatLike} from 'maplibre-gl';
  import {onMount} from 'svelte';

  export let latitude: number | null = null;
  export let longitude: number | null = null;
  export let editable: boolean = false;
  export let position: LngLatLike = {lng: 10, lat: 47};
  export let zoom: number = 6;

  // Access token
  //
  // Please be fair and don't mis-use. I know that I can move this out of the source, but it's a
  // free account anyways, so all you can do is annoy me if the account is banned.
  const MAPBOX_ACCESS_TOKEN =
    'pk.eyJ1IjoiZGFuaWxvIiwiYSI6ImNrMHk4bHcyaTA0OGMzcHA2aXloems2MnQifQ.YovfgNgeajD4aORTUE5aFw';

  // Map type
  //let mapType: 'mapbox-outdoors' | 'mapbox-satellite' | 'swisstopo' | 'swissimage' = 'mapbox-outdoors';
  let mapType = 'mapbox-outdoors';

  // Mapbox styles
  const MAPBOX_STYLE_DEFAULT = 'outdoors-v11';
  const MAPBOX_STYLE_SATELLITE = 'satellite-v9';
  const MAPBOX_STYLE_LIGHT = 'light-v10';

  // Swisstopo WMS base URL (without LAYERS)
  const SWISSTOPO_WMS_BASE_URL =
    'https://wms.geo.admin.ch/?SERVICE=WMS' +
    '&REQUEST=GetMap' +
    '&VERSION=1.3.0' +
    '&STYLES=default' +
    '&CRS=EPSG:3857' +
    '&BBOX={bbox-epsg-3857}' +
    '&WIDTH=256' +
    '&HEIGHT=256' +
    '&FORMAT=image/png';

  // Map variable
  let map: Map | null = null;
  let mapMarker: Marker | undefined;

  // Update map style whenever variable above changes
  let prevMapType = mapType;
  function updateMapType(map: Map, newMapType: typeof mapType) {
    // No-op if type did not change
    if (newMapType === prevMapType) {
      return;
    }

    // Prepare additional layers that will be added once the style is loaded
    map.once('style.load', () => {
      switch (newMapType) {
        case 'swisstopo':
          map.addLayer({
            id: 'swisstopo-layer',
            type: 'raster',
            source: {
              type: 'raster',
              tiles: [SWISSTOPO_WMS_BASE_URL + '&LAYERS=ch.swisstopo.pixelkarte-farbe'],
              tileSize: 256,
            },
          });
          break;
        case 'swissimage':
          map.addLayer({
            id: 'swissimage-layer',
            type: 'raster',
            source: {
              type: 'raster',
              tiles: [SWISSTOPO_WMS_BASE_URL + '&LAYERS=ch.swisstopo.swissimage'],
              tileSize: 256,
            },
          });
          break;
      }
    });

    // Force-set style of the MapBox base layer.
    // This will remove all existing styles and layers.
    switch (newMapType) {
      case 'mapbox-outdoors':
        map.setStyle(`mapbox://styles/mapbox/${MAPBOX_STYLE_DEFAULT}`, {diff: false});
        break;
      case 'mapbox-satellite':
        map.setStyle(`mapbox://styles/mapbox/${MAPBOX_STYLE_SATELLITE}`, {diff: false});
        break;
      case 'swisstopo':
      case 'swissimage':
        map.setStyle(`mapbox://styles/mapbox/${MAPBOX_STYLE_LIGHT}`, {diff: false});
        break;
    }

    prevMapType = newMapType;
  }

  // Handle map type updates
  $: if (map !== null) {
    updateMapType(map, mapType);
  }

  // Helper function to validate a coordinate pair
  function validPos(pos: {
    lng: number | null;
    lat: number | null;
  }): pos is {lng: number; lat: number} {
    let {lng, lat} = pos;
    if (lng === null || lat === null) {
      return false;
    }
    if (isNaN(lng) || isNaN(lat)) {
      return false;
    }
    if (lng < -180 || lng > 180) {
      return false;
    }
    if (lat < -90 || lat > 90) {
      return false;
    }
    return true;
  }

  // When the input value changes, update the marker
  $: reactive(() => {
    if (!editable || mapMarker === undefined) {
      return;
    }
    const pos = {lng: longitude, lat: latitude};
    if (validPos(pos) === true) {
      mapMarker.setLngLat(pos);
      map?.flyTo({center: pos});
    }
  }, [latitude, longitude]);

  onMount(() => {
    // Create map
    map = new Map({
      container: 'map',
      style: `mapbox://styles/mapbox/${MAPBOX_STYLE_DEFAULT}`,
      doubleClickZoom: false,
      center: position,
      zoom,
      accessToken: MAPBOX_ACCESS_TOKEN,
    });

    // Add navigation controls
    map.addControl(new NavigationControl());

    // Add draggable marker
    const marker = new Marker({draggable: editable}).setLngLat(position).addTo(map);

    if (editable) {
      // Function to update coordinates from marker
      const updateCoordinatesFromMarker = () => {
        if (!editable) {
          return;
        }
        const lngLat = marker.getLngLat();
        latitude = Number(lngLat.lat.toFixed(5));
        longitude = Number(lngLat.lng.toFixed(5));
      };

      // Update coordinates on marker drag
      marker.on('dragend', updateCoordinatesFromMarker);

      // Update marker and coordinates on double click
      map.on('dblclick', (e) => {
        marker.setLngLat(e.lngLat);
        updateCoordinatesFromMarker();
      });
    }
  });
</script>

<div id="map" class="map">
  <div class="map-style-switcher" title="Map type">
    <select bind:value={mapType}>
      <option value="mapbox-outdoors">Mapbox Outdoors</option>
      <option value="mapbox-satellite">Mapbox Satellite</option>
      <option value="swisstopo">Swisstopo</option>
      <option value="swissimage">Swissimage</option>
    </select>
  </div>
</div>
