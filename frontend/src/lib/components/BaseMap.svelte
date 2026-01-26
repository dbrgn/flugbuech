<script lang="ts">
  import {Map, NavigationControl, Marker, type LngLatLike} from 'maplibre-gl';
  import {onMount, tick} from 'svelte';

  import {unreachable} from '$lib/assert';
  import {MapDoubleClickDetector} from '$lib/map-helpers';
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

  // Minimum zoom level for elevation queries (contours available from zoom 9)
  const MIN_ZOOM_FOR_ELEVATION_QUERY = 9;

  export let mode: 'single' | 'multi';

  // Common props
  export let mapMode: 'small' | 'large' = 'small';

  // Props only used for mode 'single'
  export let center: LngLatLike = DEFAULT_MAP_CENTER;
  export let zoom: number = 6;
  export let latitude: number | null = null;
  export let longitude: number | null = null;
  export let editable: boolean = false;

  export let onElevationLookup:
    | ((data: {elevation: number | null; zoomTooLow: boolean}) => void)
    | undefined = undefined;
  export let onCountryLookup: ((data: {countryCode: string | null}) => void) | undefined =
    undefined;

  // Props only used for mode 'multi'
  export let markers: NamedCoordinates[] = [];

  // Map type
  type MapType = 'mapbox-outdoors' | 'mapbox-satellite' | 'swisstopo' | 'swissimage';
  let mapType: MapType = 'mapbox-outdoors';

  // Map variable
  let container: HTMLElement;
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
   * Query elevation at a given point using terrain contours.
   * Returns the elevation in meters, or null if not available.
   */
  function queryElevation(
    initializedMap: Map,
    lngLat: {lng: number; lat: number},
  ): {
    elevation: number | null;
    zoomTooLow: boolean;
  } {
    const currentZoom = initializedMap.getZoom();
    if (currentZoom < MIN_ZOOM_FOR_ELEVATION_QUERY) {
      return {elevation: null, zoomTooLow: true};
    }

    const point = initializedMap.project([lngLat.lng, lngLat.lat]);
    const features = initializedMap.queryRenderedFeatures(point, {
      layers: ['terrain-contours-data'],
    });

    if (features.length === 0) {
      return {elevation: null, zoomTooLow: false};
    }

    // Get the highest elevation from overlapping contour polygons
    const elevations = features
      .map((f) => f.properties?.ele as number | undefined)
      .filter((ele): ele is number => ele !== undefined);

    if (elevations.length === 0) {
      return {elevation: null, zoomTooLow: false};
    }

    return {elevation: Math.max(...elevations), zoomTooLow: false};
  }

  /**
   * Query country code at a given point.
   * Returns the ISO 3166-1 alpha-2 country code, or null if not available.
   * Country boundaries are available at all zoom levels.
   */
  function queryCountryCode(
    initializedMap: Map,
    lngLat: {lng: number; lat: number},
  ): string | null {
    const point = initializedMap.project([lngLat.lng, lngLat.lat]);
    const features = initializedMap.queryRenderedFeatures(point, {
      layers: ['countries-data'],
    });

    if (features.length === 0) {
      return null;
    }

    // Get the country code from the first feature
    return (features[0].properties?.iso_3166_1 as string | undefined) ?? null;
  }

  /**
   * Perform location data lookup (elevation and country) and dispatch events.
   */
  function performLocationDataLookup(initializedMap: Map, lngLat: {lng: number; lat: number}) {
    if (!editable) {
      return;
    }

    const elevationResult = queryElevation(initializedMap, lngLat);
    const countryCode = queryCountryCode(initializedMap, lngLat);

    onElevationLookup?.(elevationResult);
    onCountryLookup?.({countryCode});
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
          };

          // Function to update marker position, coordinates, and location lookup data
          const updateMarkerPosition = (lngLat: LngLatLike) => {
            marker.setLngLat(lngLat);
            ensureSingleMarkerVisible();
            updateCoordinatesFromMarker();
            // Lookup elevation and country code
            const markerLngLat = marker.getLngLat();
            performLocationDataLookup(initializedMap, {
              lng: markerLngLat.lng,
              lat: markerLngLat.lat,
            });
          };

          // Update coordinates and lookup data on marker drag
          marker.on('dragend', () => {
            updateCoordinatesFromMarker();
            const markerLngLat = marker.getLngLat();
            performLocationDataLookup(initializedMap, {
              lng: markerLngLat.lng,
              lat: markerLngLat.lat,
            });
          });

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

      // Add invisible data layers for location lookups (elevation and country code).
      // Only loaded when adding/editing a location.
      if (editable) {
        // Terrain contours for elevation lookup
        initializedMap.addLayer({
          'id': 'terrain-contours-data',
          'type': 'fill',
          'source': {
            type: 'vector',
            url: `https://api.mapbox.com/v4/mapbox.mapbox-terrain-v2.json?access_token=${MAPBOX_ACCESS_TOKEN}`,
          },
          'source-layer': 'contour',
          'paint': {
            'fill-opacity': 0,
          },
        });

        // Country boundaries for country code lookup
        initializedMap.addLayer({
          'id': 'countries-data',
          'type': 'fill',
          'source': {
            type: 'vector',
            url: `https://api.mapbox.com/v4/mapbox.country-boundaries-v1.json?access_token=${MAPBOX_ACCESS_TOKEN}`,
          },
          'source-layer': 'country_boundaries',
          'paint': {
            'fill-opacity': 0,
          },
        });
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
      container,
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
