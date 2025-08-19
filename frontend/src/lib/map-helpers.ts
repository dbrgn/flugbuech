import type {LngLatLike, Map, MapMouseEvent} from 'maplibre-gl';

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
