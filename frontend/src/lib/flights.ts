import type {i18n as I18nextType} from 'i18next';

/**
 * Return a verbose translated name for the specified flight.
 */
export function flightName(
    flight: {
        id: number;
        number?: number;
        launchAt?: string;
        landingAt?: string;
    },
    i18n: I18nextType,
): string {
    let name =
        flight.number !== undefined
            ? i18n.t('flight.snippet--flight-with-number', {
                  number: flight.number,
              })
            : i18n.t('flight.snippet--flight-with-id', {id: flight.id});
    if (flight.launchAt !== undefined) {
        name += ' ';
        name += i18n.t('flight.snippet--from', {location: flight.launchAt});
    }
    if (flight.landingAt !== undefined) {
        name += ' ';
        name += i18n.t('flight.snippet--to', {location: flight.landingAt});
    }
    return name;
}
