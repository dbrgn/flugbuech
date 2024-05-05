<script lang="ts">
  import {Map, NavigationControl, Marker, type LngLatLike} from 'maplibre-gl';
  import {onMount} from 'svelte';

  import {unreachable} from '$lib/assert';
  import {reactive} from '$lib/svelte';

  import {
    DEFAULT_MAP_CENTER,
    MAPBOX_ACCESS_TOKEN,
    MAPBOX_STYLE_DEFAULT,
    MAPBOX_STYLE_LIGHT,
    MAPBOX_STYLE_SATELLITE,
    MARKER_COLOR,
    SWISSTOPO_WMS_BASE_URL,
    isValidPos,
    type NamedCoordinates,
  } from './map';

  export let mode: 'single' | 'multi';

  // Props only used for mode 'single'
  export let center: LngLatLike = DEFAULT_MAP_CENTER;
  export let zoom: number = 6;
  export let latitude: number | null = null;
  export let longitude: number | null = null;
  export let editable: boolean = false;

  // Props only used for mode 'multi'
  export let markers: NamedCoordinates[] = [];

  // Map type
  type MapType = 'mapbox-outdoors' | 'mapbox-satellite' | 'swisstopo' | 'swissimage';
  let mapType: MapType = 'mapbox-outdoors';

  // Map variable
  let map: Map | null = null;

  // Markers
  let mapMarker: Marker | undefined;
  let markersLoaded = false;

  /**
   * If the single map marker wasn't yet added to the map, do that now.
   */
  function ensureSingleMarkerVisible() {
    if (map !== null && mapMarker !== undefined && !markersLoaded) {
      mapMarker.addTo(map);
      markersLoaded = true;
    }
  }

  /**
   * Function to add markers and labels to map.
   *
   * Note: Markers don't need to be re-loaded on style change, labels do need to be re-loaded.
   */
  function addMapMarkersAndLabels(initializedMap: Map) {
    switch (mode) {
      case 'single': {
        if (mapMarker !== undefined) {
          return;
        }

        // Add marker (draggable if editable)
        const marker = new Marker({color: MARKER_COLOR, draggable: editable}).setLngLat(center);

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
          initializedMap.on('dblclick', (e) => {
            marker.setLngLat(e.lngLat);
            ensureSingleMarkerVisible();
            updateCoordinatesFromMarker();
          });
        }

        // Update reference
        mapMarker = marker;

        // Initialize marker (if coordinates are available)
        if (!markersLoaded && (!editable || (latitude !== null && longitude !== null))) {
          ensureSingleMarkerVisible();
        }

        break;
      }
      case 'multi': {
        // Add markers
        if (!markersLoaded) {
          for (const coords of markers) {
            new Marker({color: MARKER_COLOR, scale: 0.8}).setLngLat(coords).addTo(initializedMap);
          }
        }

        // Add marker labels
        initializedMap.addSource('locations', {
          type: 'geojson',
          data: {
            type: 'FeatureCollection',
            features: markers.map((coords) => ({
              type: 'Feature',
              geometry: {type: 'Point', coordinates: [coords.lon, coords.lat]},
              properties: {
                title: coords.name,
              },
            })),
          },
        });
        initializedMap.addLayer({
          id: 'locations',
          type: 'symbol',
          source: 'locations',
          minzoom: 9,
          layout: {
            // Get the title name from the source's "title" property
            'text-field': ['get', 'title'],
            'text-font': ['Open Sans Semibold', 'Arial Unicode MS Bold'],
            'text-offset': [0, 0.25],
            'text-anchor': 'top',
            'text-size': 14,
          },
          paint: {
            'text-halo-width': 2,
            'text-halo-color': 'white',
          },
        });

        // Apply bounding box
        if (!markersLoaded) {
          if (markers.length > 0) {
            const longitudes = markers.map((coord) => coord.lon);
            const latitudes = markers.map((coord) => coord.lat);

            const minLon = Math.min(...longitudes);
            const maxLon = Math.max(...longitudes);
            const minLat = Math.min(...latitudes);
            const maxLat = Math.max(...latitudes);

            initializedMap.fitBounds(
              [
                [minLon, minLat], // SW
                [maxLon, maxLat], // NE
              ],
              {padding: 40, maxZoom: 11},
            );
          }
        }

        markersLoaded = true;

        break;
      }
      default:
        unreachable(mode);
    }
  }

  /**
   * Update map style whenever {@link mapType} variable changes.
   */
  let prevMapType: MapType | undefined;
  function updateMapType(initializedMap: Map, newMapType: MapType) {
    // No-op if type did not change
    if (newMapType === prevMapType) {
      return;
    }

    // Prepare additional layers that will be added once the style is loaded
    initializedMap.once('style.load', () => {
      // Additional map layers on top of base style
      switch (newMapType) {
        case 'swisstopo':
          initializedMap.addLayer({
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
          initializedMap.addLayer({
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

      // Map markers and labels
      addMapMarkersAndLabels(initializedMap);
    });

    // Force-set style of the MapBox base layer.
    // This will remove all existing styles and layers.
    switch (newMapType) {
      case 'mapbox-outdoors':
        initializedMap.setStyle(`mapbox://styles/mapbox/${MAPBOX_STYLE_DEFAULT}`, {diff: false});
        break;
      case 'mapbox-satellite':
        initializedMap.setStyle(`mapbox://styles/mapbox/${MAPBOX_STYLE_SATELLITE}`, {diff: false});
        break;
      case 'swisstopo':
      case 'swissimage':
        initializedMap.setStyle(`mapbox://styles/mapbox/${MAPBOX_STYLE_LIGHT}`, {diff: false});
        break;
    }

    prevMapType = newMapType;
  }

  // Handle map type updates
  $: if (map !== null) {
    updateMapType(map, mapType);
  }

  // In 'single' mode, when the input value changes, update the marker
  $: reactive(() => {
    if (mode !== 'single' || !editable || mapMarker === undefined) {
      return;
    }
    const pos = {lng: longitude, lat: latitude};
    if (isValidPos(pos) === true) {
      mapMarker.setLngLat(pos);
      ensureSingleMarkerVisible();
      map?.flyTo({center: pos});
    }
  }, [latitude, longitude]);

  onMount(() => {
    // Create map
    map = new Map({
      container: 'map',
      style: `mapbox://styles/mapbox/${MAPBOX_STYLE_DEFAULT}`,
      doubleClickZoom: !editable,
      center: center,
      zoom,
      accessToken: MAPBOX_ACCESS_TOKEN,
    });

    // Add navigation controls
    map.addControl(new NavigationControl());
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
