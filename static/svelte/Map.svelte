<script>
    import mapboxgl from 'mapbox-gl';
    import { onMount } from 'svelte';

    // Props
    export let lngInput;
    export let latInput;
    export let position = {lng: 10, lat: 47};
    export let zoom = 6;

    const STYLE_DEFAULT = 'outdoors-v11';
    const STYLE_SATELLITE = 'satellite-v9';
    const ICON_MAP = 'fa-map';
    const ICON_SATELLITE = 'fa-satellite';

    // Configure map token
    mapboxgl.accessToken = 'pk.eyJ1IjoiZGFuaWxvIiwiYSI6ImNrMHk4bHcyaTA0OGMzcHA2aXloems2MnQifQ.YovfgNgeajD4aORTUE5aFw';

    // Map variable
    let map = null;

    // Current style
    let style = STYLE_DEFAULT;

    // Update map style whenever variable above changes
    $: if (map !== null) {
        map.setStyle(`mapbox://styles/mapbox/${style}`);
    }

    onMount(() => {
        // Create map
        map = new mapboxgl.Map({
            container: 'map',
            style: `mapbox://styles/mapbox/${style}`,
            doubleClickZoom: false,
            center: position,
            zoom: zoom,
        });

        // Add navigation controls
        map.addControl(new mapboxgl.NavigationControl());

        // Add draggable marker
        const marker = new mapboxgl.Marker({draggable: true})
            .setLngLat(position)
            .addTo(map);

        // Function to update coordinates from marker
        const updateCoordinatesFromMarker = () => {
            const lngLat = marker.getLngLat();
            lngInput.value = lngLat.lng.toFixed(5);
            latInput.value = lngLat.lat.toFixed(5);
        };

        // Update coordinates on marker drag
        marker.on('dragend', updateCoordinatesFromMarker);

        // Update marker and coordinates on double click
        map.on('dblclick', (e) => {
            marker.setLngLat(e.lngLat)
            updateCoordinatesFromMarker();
        });
    });
</script>

<div id="map" class="map">
    {#if style !== STYLE_SATELLITE}
    <div class="map-style-switcher switch-to-satellite icon" title="Switch to satellite" on:click={() => style = STYLE_SATELLITE}>
        <i class="fas fa-satellite"></i>
    </div>
    {/if}
    {#if style !== STYLE_DEFAULT}
    <div class="map-style-switcher switch-to-default icon" title="Switch to default map" on:click={() => style = STYLE_DEFAULT}>
        <i class="fas fa-map"></i>
    </div>
    {/if}
</div>
