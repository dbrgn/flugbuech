<script lang="typescript">
    import { Map, NavigationControl, Marker, LngLatLike } from 'maplibre-gl';
    import { onMount } from 'svelte';

    // Props
    export let lngInput: HTMLInputElement;
    export let latInput: HTMLInputElement;
    export let position: LngLatLike = {lng: 10, lat: 47};
    export let zoom: number = 6;

    // Style
    const STYLE_DEFAULT = 'outdoors-v11';
    const STYLE_SATELLITE = 'satellite-v9';
    let style = STYLE_DEFAULT;

    // Access token
    const MAPBOX_ACCESS_TOKEN = 'pk.eyJ1IjoiZGFuaWxvIiwiYSI6ImNrMHk4bHcyaTA0OGMzcHA2aXloems2MnQifQ.YovfgNgeajD4aORTUE5aFw';

    // Map variable
    let map: Map | null = null;

    // Update map style whenever variable above changes
    $: if (map !== null) {
        map.setStyle(`mapbox://styles/mapbox/${style}`);
    }

    // Helper function to validate a coordinate pair
    function validLngLat(lng: number | null, lat: number | null): boolean {
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

    onMount(() => {
        // Create map
        map = new Map({
            container: 'map',
            style: `mapbox://styles/mapbox/${style}`,
            doubleClickZoom: false,
            center: position,
            zoom: zoom,
            accessToken: MAPBOX_ACCESS_TOKEN,
        });

        // Add navigation controls
        map.addControl(new NavigationControl());

        // Add draggable marker
        const marker = new Marker({draggable: true})
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
            marker.setLngLat(e.lngLat);
            updateCoordinatesFromMarker();
        });

        // When the input value changes, update the marker
        const updateMarkerFromCoordinates = () => {
            const lng = parseFloat(lngInput.value);
            const lat = parseFloat(latInput.value);
            if (validLngLat(lng, lat) === true) {
                const pos = {lng: lng, lat: lat};
                marker.setLngLat(pos);
                map.flyTo({center: pos});
            }
        };
        lngInput.addEventListener('change', updateMarkerFromCoordinates);
        latInput.addEventListener('change', updateMarkerFromCoordinates);
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
