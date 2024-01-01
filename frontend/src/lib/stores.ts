import {writable} from 'svelte/store';

export interface Flash {
    readonly message: string;
    readonly severity: 'info' | 'success' | 'warning' | 'error';
    readonly icon?: `fa-${string}`;
}

export const flashes = writable<Flash[]>([]);

export function addFlash(flash: Flash): void {
    flashes.update((f) => [...f, flash]);
}
