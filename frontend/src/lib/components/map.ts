// Mapbox access token
//
// Please be fair and don't mis-use. I know that I can move this out of the source, but it's a
// free account anyways, so all you can do is annoy me if the account is banned.
export const MAPBOX_ACCESS_TOKEN =
    'pk.eyJ1IjoiZGFuaWxvIiwiYSI6ImNrMHk4bHcyaTA0OGMzcHA2aXloems2MnQifQ.YovfgNgeajD4aORTUE5aFw';

// Mapbox styles
export const MAPBOX_STYLE_DEFAULT = 'outdoors-v11';
export const MAPBOX_STYLE_SATELLITE = 'satellite-v9';
export const MAPBOX_STYLE_LIGHT = 'light-v10';

// Swisstopo WMS base URL (without LAYERS)
export const SWISSTOPO_WMS_BASE_URL =
    'https://wms.geo.admin.ch/?SERVICE=WMS' +
    '&REQUEST=GetMap' +
    '&VERSION=1.3.0' +
    '&STYLES=default' +
    '&CRS=EPSG:3857' +
    '&BBOX={bbox-epsg-3857}' +
    '&WIDTH=256' +
    '&HEIGHT=256' +
    '&FORMAT=image/png';

// The default centerpoint of the map
export const DEFAULT_MAP_CENTER = {lng: 10, lat: 47};

export const MARKER_COLOR = '#1496ED'; // Primary color blue

export interface Coordinates {
    readonly lon: number;
    readonly lat: number;
}

export interface NamedCoordinates extends Coordinates {
    readonly name: string;
}

// Helper function to validate a coordinate pair
export function isValidPos(pos: {
    lng: number | null;
    lat: number | null;
}): pos is {lng: number; lat: number} {
    const {lng, lat} = pos;
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
