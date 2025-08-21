<script lang="ts">
  import {Map, NavigationControl, Marker, type LngLatLike} from 'maplibre-gl';
  import {onMount, tick} from 'svelte';

  import {unreachable} from '$lib/assert';
  import {
    MapDoubleClickDetector,
    queryCountryAtPoint,
    queryElevationAtPoint,
  } from '$lib/map-helpers';
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

  // Map heights
  const MAP_HEIGHT_SMALL = '400px';
  const MAP_HEIGHT_LARGE = 'max(100vh, 800px)';

  export let mode: 'single' | 'multi';

  // Common props
  export let mapMode: 'small' | 'large' = 'small';

  // Props only used for mode 'single'
  export let center: LngLatLike = DEFAULT_MAP_CENTER;
  export let zoom: number = 6;
  export let latitude: number | null = null;
  export let longitude: number | null = null;
  export let editable: boolean = false;
  export let onMarkerChange:
    | ((info: {
        lng: number;
        lat: number;
        countryCode: string | null;
        elevation: number | null;
      }) => void)
    | undefined = undefined;

  // Props only used for mode 'multi'
  export let markers: NamedCoordinates[] = [];

  // Map type
  type MapType = 'mapbox-outdoors' | 'mapbox-satellite' | 'swisstopo' | 'swissimage';
  let mapType: MapType = 'mapbox-outdoors';

  // Map variable
  let container: HTMLElement;
  let map: Map | null = null;
  let mapStyleLoaded = false;

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
   * Toggle map height.
   */
  function toggleMapSize() {
    mapMode = mapMode === 'small' ? 'large' : 'small';
    tick().then(() => map?.resize());
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

            // Query country and elevation at new position and call marker change callback
            if (map && onMarkerChange) {
              const elevation = queryElevationAtPoint(map, lngLat.lng, lngLat.lat);
              const countryCode = queryCountryAtPoint(map, lngLat.lng, lngLat.lat);

              onMarkerChange({
                lng: lngLat.lng,
                lat: lngLat.lat,
                countryCode,
                elevation,
              });
            }
          };

          // Function to update marker position and coordinates
          const updateMarkerPosition = (lngLat: LngLatLike) => {
            marker.setLngLat(lngLat);
            ensureSingleMarkerVisible();
            updateCoordinatesFromMarker();
          };

          // Update coordinates on marker drag
          marker.on('dragend', updateCoordinatesFromMarker);

          // Set up double click detection, update marker and coordinates on double click (desktop)
          // or double tap (mobile)
          const doubleClickDetector = new MapDoubleClickDetector(initializedMap, (lngLat) =>
            updateMarkerPosition(lngLat),
          );
          doubleClickDetector.registerEvents();
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

    // Check if style is loaded before proceeding
    if (!mapStyleLoaded) {
      return;
    }

    try {
      // Remove existing layers safely
      const layersToRemove = [
        'mapbox-raster-layer',
        'mapbox-light-base-layer',
        'swisstopo-layer',
        'swissimage-layer',
        'countries-layer',
      ];
      const sourcesToRemove = [
        'mapbox-raster',
        'mapbox-light-base',
        'swisstopo-source',
        'swissimage-source',
        'countries-source',
        'terrain-rgb-source',
      ];

      layersToRemove.forEach((layerId) => {
        if (initializedMap.getLayer(layerId)) {
          initializedMap.removeLayer(layerId);
        }
      });

      sourcesToRemove.forEach((sourceId) => {
        if (initializedMap.getSource(sourceId)) {
          initializedMap.removeSource(sourceId);
        }
      });

      // Add the appropriate map layer
      switch (newMapType) {
        case 'mapbox-outdoors':
          initializedMap.addSource('mapbox-raster', {
            type: 'raster',
            tiles: [
              `https://api.mapbox.com/styles/v1/mapbox/${MAPBOX_STYLE_DEFAULT}/tiles/{z}/{x}/{y}?access_token=${MAPBOX_ACCESS_TOKEN}`,
            ],
            tileSize: 512,
          });
          initializedMap.addLayer({
            id: 'mapbox-raster-layer',
            type: 'raster',
            source: 'mapbox-raster',
          });
          break;
        case 'mapbox-satellite':
          initializedMap.addSource('mapbox-raster', {
            type: 'raster',
            tiles: [
              `https://api.mapbox.com/styles/v1/mapbox/${MAPBOX_STYLE_SATELLITE}/tiles/{z}/{x}/{y}?access_token=${MAPBOX_ACCESS_TOKEN}`,
            ],
            tileSize: 512,
          });
          initializedMap.addLayer({
            id: 'mapbox-raster-layer',
            type: 'raster',
            source: 'mapbox-raster',
          });
          break;
        case 'swisstopo':
          initializedMap.addSource('mapbox-light-base', {
            type: 'raster',
            tiles: [
              `https://api.mapbox.com/styles/v1/mapbox/${MAPBOX_STYLE_LIGHT}/tiles/{z}/{x}/{y}?access_token=${MAPBOX_ACCESS_TOKEN}`,
            ],
            tileSize: 512,
          });
          initializedMap.addLayer({
            id: 'mapbox-light-base-layer',
            type: 'raster',
            source: 'mapbox-light-base',
          });
          initializedMap.addSource('swisstopo-source', {
            type: 'raster',
            tiles: [SWISSTOPO_WMS_BASE_URL + '&LAYERS=ch.swisstopo.pixelkarte-farbe'],
            tileSize: 256,
          });
          initializedMap.addLayer({
            id: 'swisstopo-layer',
            type: 'raster',
            source: 'swisstopo-source',
          });
          break;
        case 'swissimage':
          initializedMap.addSource('mapbox-light-base', {
            type: 'raster',
            tiles: [
              `https://api.mapbox.com/styles/v1/mapbox/${MAPBOX_STYLE_LIGHT}/tiles/{z}/{x}/{y}?access_token=${MAPBOX_ACCESS_TOKEN}`,
            ],
            tileSize: 512,
          });
          initializedMap.addLayer({
            id: 'mapbox-light-base-layer',
            type: 'raster',
            source: 'mapbox-light-base',
          });
          initializedMap.addSource('swissimage-source', {
            type: 'raster',
            tiles: [SWISSTOPO_WMS_BASE_URL + '&LAYERS=ch.swisstopo.swissimage'],
            tileSize: 256,
          });
          initializedMap.addLayer({
            id: 'swissimage-layer',
            type: 'raster',
            source: 'swissimage-source',
          });
          break;
      }

      // Always add the countries source for country code detection (but hidden)
      if (onMarkerChange && !initializedMap.getSource('countries-source')) {
        initializedMap.addSource('countries-source', {
          type: 'vector',
          tiles: [
            `https://api.mapbox.com/v4/mapbox.country-boundaries-v1/{z}/{x}/{y}.vector.pbf?access_token=${MAPBOX_ACCESS_TOKEN}`,
          ],
          minzoom: 0,
          maxzoom: 8,
        });

        initializedMap.addLayer({
          'id': 'countries-layer',
          'type': 'fill',
          'source': 'countries-source',
          'source-layer': 'country_boundaries',
          'paint': {
            'fill-opacity': 0, // Make it invisible
          },
          'filter': [
            'all',
            ['==', ['get', 'disputed'], 'false'],
            ['any', ['==', 'all', ['get', 'worldview']], ['in', 'US', ['get', 'worldview']]],
          ],
        });
      }

      if (onMarkerChange && !initializedMap.getSource('terrain-rgb-source')) {
        initializedMap.addSource('terrain-rgb-source', {
          type: 'raster-dem',
          tiles: [
            `https://api.mapbox.com/v4/mapbox.terrain-rgb/{z}/{x}/{y}.pngraw?access_token=${MAPBOX_ACCESS_TOKEN}`,
          ],
          tileSize: 256,
          maxzoom: 15,
          encoding: 'mapbox',
        });

        initializedMap.setTerrain({
          source: 'terrain-rgb-source',
          exaggeration: 1,
        });
      }

      prevMapType = newMapType;
    } catch (error) {
      console.error('Error updating map type:', error);
    }
  }

  // Handle map type updates
  $: if (map !== null && mapStyleLoaded) {
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

      // Query country and elevation at new position
      if (map && onMarkerChange) {
        // Query both synchronously
        const elevation = queryElevationAtPoint(map, pos.lng, pos.lat);
        const countryCode = queryCountryAtPoint(map, pos.lng, pos.lat);

        onMarkerChange({
          lng: pos.lng,
          lat: pos.lat,
          countryCode,
          elevation,
        });
      }
    }
  }, [latitude, longitude]);

  onMount(() => {
    // Create map with a blank base style
    map = new Map({
      container,
      style: {
        version: 8,
        sources: {},
        layers: [],
      },
      doubleClickZoom: !editable,
      center: center,
      zoom,
    });

    // Add navigation controls
    map.addControl(new NavigationControl());

    // Once the style is loaded, add the initial map layer
    map.on('style.load', () => {
      if (map) {
        mapStyleLoaded = true;

        // Reset to ensure initial update works
        prevMapType = undefined;
        updateMapType(map, mapType);

        // Add markers and labels
        addMapMarkersAndLabels(map);
      }
    });
  });
</script>

<div
  class="map"
  bind:this={container}
  style:height={mapMode === 'small' ? MAP_HEIGHT_SMALL : MAP_HEIGHT_LARGE}
>
  <button type="button" class="map-resize-button button" on:click={toggleMapSize}>
    <span class="icon">
      {#if mapMode === 'small'}
        <i class="fa-solid fa-up-right-and-down-left-from-center"></i>
      {:else}
        <i class="fa-solid fa-down-left-and-up-right-to-center"></i>
      {/if}
    </span>
  </button>
  <div class="map-style-switcher" title="Map type">
    <select bind:value={mapType}>
      <option value="mapbox-outdoors">Mapbox Outdoors</option>
      <option value="mapbox-satellite">Mapbox Satellite</option>
      <option value="swisstopo">Swisstopo</option>
      <option value="swissimage">Swissimage</option>
    </select>
  </div>
</div>

<style>
  .map {
    position: relative;
  }

  .map-style-switcher {
    position: absolute;
    top: 0;
    left: 0;
    padding: 8px;
    z-index: 9999;
  }

  .map-style-switcher select {
    font-size: 14px;
  }

  .map-resize-button {
    position: absolute;
    top: 110px;
    right: 10px;
    width: 29px;
    height: 29px;
    z-index: 9999;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.1);
  }
</style>
