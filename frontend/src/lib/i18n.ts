import i18next, {type i18n as I18nextType} from 'i18next';
import ICU from 'i18next-icu';
import {derived, get, writable, type Readable, type Writable} from 'svelte/store';

import {keys} from '$lib/object';
import type {StrictPartial} from '$lib/types';
import translationDeJson from '$translations/de/translation.json';
import translationEnJson from '$translations/en/translation.json';

/**
 * Define English as the base translation. All other translations will only be able to (optionally)
 * provide keys defined by the base translation.
 */
type BaseTranslation = typeof translationEnJson;

// Assign translations
const translationEn: BaseTranslation = translationEnJson;

// Casting the `translation*Json` (other than the base `translationEnJson`) values imported from the
// JSON files as `StrictPartial` of `BaseTranslation` ensures that all translations provide only
// keys defined in the base translation while allowing for missing keys. If a translation provides a
// key that does not exist in the base translation, a type error is raised here when typechecking.
const translationDe: StrictPartial<typeof translationDeJson, BaseTranslation> = translationDeJson;

// Consider keeping the locales in sync in the i18next-parser.config.js file.
export const resources = {
    de: {translation: translationDe},
    en: {translation: translationEn},
} as const;

/**
 * Available locales.
 */
export const LOCALES = keys(resources);

/**
 * Mapping from locale identifier to name in that language.
 *
 * Can be used for language switchers.
 */
export const LOCALE_NAMES: {[Locale in keyof typeof resources]: string} = {
    de: 'Deutsch',
    en: 'English',
};

export type Locale = (typeof LOCALES)[number];

const FALLBACK_LOCALE: Locale = 'en' as const;

/**
 * Determine the browser's most preferred locale that we support, or fall back to the default locale
 * otherwise.
 */
export function determineBrowserLocale(): Locale {
    console.debug('i18n: Determine browser locale...');
    if (typeof window !== 'undefined' && window.navigator.languages !== undefined) {
        for (const language of navigator.languages) {
            const locale = getClosestAvailableLocale(language);
            if (locale !== undefined) {
                console.debug(
                    `i18n: Found supported language "${locale}" for browser locale "${language}"`,
                );
                return locale;
            }
            console.debug(`i18n: Skipping browser locale "${language}", not supported`);
        }
    }
    console.debug(`i18n: Falling back to "${FALLBACK_LOCALE}"`);
    return FALLBACK_LOCALE;
}

export function isLocale(locale: string): locale is Locale {
    return (LOCALES as readonly string[]).includes(locale);
}

function getClosestAvailableLocale(locale: string): Locale | undefined {
    // If specified string is a supported locale, use it directly
    if (isLocale(locale)) {
        return locale;
    }

    // If the primary language subtag associated with the specified locale string is a supported
    // locale, use it.
    //
    // Example: If the specified locale string is "de_CH", use the "de" locale.
    try {
        const minimizedLocale = new Intl.Locale(locale).language;
        if (isLocale(minimizedLocale)) {
            return minimizedLocale;
        }
    } catch (error) {
        // Unable to create an Intl.Locale object from locale.
        // Ignoring error.
    }

    // Give up
    return undefined;
}

function createI18nStore(i18n: I18nextType): Writable<{i18n: I18nextType}> {
    const i18nStore = writable<{i18n: I18nextType}>({i18n});

    function forceStoreRefresh(): void {
        i18nStore.set({i18n});
    }

    i18n.on('initialized', forceStoreRefresh);
    i18n.on('loaded', forceStoreRefresh);
    i18n.on('added', forceStoreRefresh);
    i18n.on('languageChanged', forceStoreRefresh);

    return i18nStore;
}

const i18nStore = createI18nStore(i18next);

export async function initialize(localeStore: Readable<Locale>): Promise<void> {
    const i18n = get(i18nStore).i18n;

    if (i18n.isInitialized) {
        console.warn('Already initialized');
        return;
    }

    console.info('Initializing i18n...');

    await i18n
        .use({type: 'logger'})
        .use(ICU)
        .init({
            lng: get(localeStore),
            resources,
            fallbackLng: FALLBACK_LOCALE,
            debug: import.meta.env.DEBUG,
            returnNull: false,
        });

    console.info('i18n initialization complete', {
        language: i18n.language,
        resolvedLanguage: i18n.resolvedLanguage,
        loadedLanguages: i18n.languages,
    });

    // Note: We can ignore the unsubscriber because we will maintain a global reference to the store
    localeStore.subscribe((locale) => {
        if (isLocale(locale)) {
            if (i18n.language === locale) {
                return;
            }
            i18n.changeLanguage(locale).catch((error) => {
                throw new Error(`Changing language failed: ${error}`);
            });
        }
    });
}

export const i18n = derived(i18nStore, (value) => value.i18n);
