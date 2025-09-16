import type {LngLatLike, Map, MapMouseEvent} from 'maplibre-gl';

// Terrain exaggeration factor used for minimal visual impact while keeping elevation data accessible
const TERRAIN_EXAGGERATION = 0.01;

/**
 * Query the country at a specific coordinate and return the result.
 *
 * @param map The MapLibre GL map instance
 * @param lng Longitude coordinate
 * @param lat Latitude coordinate
 * @returns The detected country code or null if not found
 */
export function queryCountryAtPoint(map: Map, lng: number, lat: number): string | null {
    // Convert coordinates to screen point for querying
    const point = map.project([lng, lat]);

    // Query the countries layer at the point
    const features = map.queryRenderedFeatures(point, {
        layers: ['countries-layer'],
    });

    if (features.length > 0) {
        const countryCode = features[0].properties?.iso_3166_1;
        if (countryCode && typeof countryCode === 'string') {
            return countryCode;
        }
    }
    return null;
}

/**
 * Query the terrain elevation at a specific coordinate using MapLibre's built-in queryTerrainElevation.
 *
 * @param map The MapLibre GL map instance
 * @param lng Longitude coordinate
 * @param lat Latitude coordinate
 * @returns The elevation in meters or null if not available
 */
export function queryElevationAtPoint(map: Map, lng: number, lat: number): number | null {
    try {
        // Check if terrain is enabled
        if (!map.getTerrain()) {
            return null;
        }

        const elevationWithExaggeration = map.queryTerrainElevation([lng, lat]);

        if (elevationWithExaggeration === null || isNaN(elevationWithExaggeration)) {
            return null;
        }

        //The queryTerrainElevation function is designed for 3D rendering, we need to scale it inversely with the exaggeration factor
        const actualElevation = elevationWithExaggeration / TERRAIN_EXAGGERATION;
        return Math.round(actualElevation);
    } catch (error) {
        console.error('Error querying terrain elevation with native method:', error);
        return null;
    }
}

// Export the terrain exaggeration constant for use in BaseMap.svelte
export {TERRAIN_EXAGGERATION};

export interface DoubleClickDetectorOptions {
    /** Max delay for a double click/tap to be detected. */
    maxDoubleTapDelayMs: number;
    /** Max distance for a double click/tap to be detected. */
    maxDoubleTapDistancePx: number;
}

const DEFAULT_OPTIONS = {maxDoubleTapDelayMs: 400, maxDoubleTapDistancePx: 20};

/**
 * Detect double taps on a map.
 *
 * We cannot fully rely on the `dblclick` event, because it does not work properly on mobile
 * devices.
 */
export class MapDoubleClickDetector {
    private lastClickTime: number = 0;
    private lastClickCoords: {lng: number; lat: number} | undefined;

    // Event listeners
    private _clickListener: (e: MapMouseEvent) => void;
    private _doubleClickListener: (e: MapMouseEvent) => void;

    public constructor(
        private readonly _map: Map,
        private readonly _onDoubleClick: (lngLat: LngLatLike) => void,
        private readonly _options: DoubleClickDetectorOptions = DEFAULT_OPTIONS,
    ) {
        this._clickListener = (e) => this._handleClick(e);
        this._doubleClickListener = (e) => this._onDoubleClick(e.lngLat);
    }

    /**
     * Handle a click event and try to detect whether it could be part of a double-click /
     * double-tap.
     */
    private _handleClick(e: MapMouseEvent) {
        const currentTime = Date.now();
        const timeDiff = currentTime - this.lastClickTime;

        // Check if this could be a double-tap
        if (timeDiff < this._options.maxDoubleTapDelayMs && this.lastClickCoords) {
            const currentPixel = this._map.project(e.lngLat);
            const lastPixel = this._map.project(this.lastClickCoords);

            // Get distance using Pythagorean theorem
            const distance = Math.sqrt(
                Math.pow(currentPixel.x - lastPixel.x, 2) +
                    Math.pow(currentPixel.y - lastPixel.y, 2),
            );

            if (distance < this._options.maxDoubleTapDistancePx) {
                this._onDoubleClick(e.lngLat);
                // Reset to prevent triple-tap from triggering
                this.lastClickTime = 0;
                this.lastClickCoords = undefined;
                return;
            }
        }

        // Store this click for potential double-tap detection
        this.lastClickTime = currentTime;
        this.lastClickCoords = {lng: e.lngLat.lng, lat: e.lngLat.lat};
    }

    /**
     * Register event listeners.
     */
    public registerEvents() {
        this._map.on('dblclick', this._doubleClickListener);
        this._map.on('click', this._clickListener);
    }

    /**
     * Unregister event listeners.
     */
    public unregisterEvents() {
        this._map.off('dblclick', this._doubleClickListener);
        this._map.off('click', this._clickListener);
    }
}
