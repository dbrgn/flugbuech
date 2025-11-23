<script lang="ts">
  import {onMount} from 'svelte';

  import {apiPost, extractResponseError} from '$lib/api';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import SingleMap from '$lib/components/SingleMap.svelte';
  import {i18n} from '$lib/i18n';
  import {addFlash} from '$lib/stores';
  import {reactive} from '$lib/svelte';

  import {goto} from '$app/navigation';

  import type {Location} from './api';

  // Props
  export let location: Location | undefined = undefined;

  // Form values
  let name: string = location?.name ?? '';
  let countryCode: string = location?.countryCode ?? '';
  let elevation: number | null = location?.elevation ?? null;
  let latitude: number | null = location?.coordinates?.lat ?? null;
  let longitude: number | null = location?.coordinates?.lon ?? null;

  // Input transformations
  $: if (countryCode.length > 0) {
    countryCode = countryCode.toLocaleUpperCase();
  }

  // Element references
  let longitudeInput: HTMLInputElement | null;
  let latitudeInput: HTMLInputElement | null;

  // Marker change callback (includes country code and elevation detection)
  function handleMarkerChange(info: {
    lng: number;
    lat: number;
    countryCode: string | null;
    elevation: number | null;
  }) {
    // Auto-fill if changes detected
    if (info.countryCode && info.countryCode !== countryCode) {
      countryCode = info.countryCode;
    }
    if (info.elevation !== null && info.elevation !== elevation) {
      elevation = info.elevation;
    }
  }

  // Validation
  const fields = ['name', 'countryCode', 'elevation', 'latitude', 'longitude'] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    name: undefined,
    countryCode: undefined,
    elevation: undefined,
    latitude: undefined,
    longitude: undefined,
  };
  function validateName(): void {
    fieldErrors = {
      ...fieldErrors,
      name: name.length < 1 ? $i18n.t('location.error--name-empty') : undefined,
    };
  }
  $: reactive(validateName, [name]);
  function validateCountryCode(): void {
    fieldErrors = {
      ...fieldErrors,
      countryCode:
        countryCode.match(/^[A-Z]{2}$/u) === null
          ? $i18n.t('location.error--country-code-2-letters')
          : undefined,
    };
  }
  $: reactive(validateCountryCode, [countryCode]);
  function validateElevation(): void {
    fieldErrors = {
      ...fieldErrors,
      elevation: elevation === null ? $i18n.t('location.error--elevation-empty') : undefined,
    };
  }
  $: reactive(validateElevation, [elevation]);
  function validateLatitude(): void {
    fieldErrors = {
      ...fieldErrors,
      latitude:
        longitude !== null && latitude === null
          ? $i18n.t('location.error--latitude-empty')
          : undefined,
    };
  }
  $: reactive(validateLatitude, [latitude, longitude]);
  function validateLongitude(): void {
    fieldErrors = {
      ...fieldErrors,
      longitude:
        latitude !== null && longitude === null
          ? $i18n.t('location.error--longitude-empty')
          : undefined,
    };
  }
  $: reactive(validateLongitude, [latitude, longitude]);
  function validateAll(): void {
    validateName();
    validateCountryCode();
    validateElevation();
    validateLatitude();
    validateLongitude();
  }
  function resetErrors(): void {
    for (const field of fields) {
      fieldErrors[field] = undefined;
    }
  }

  // Error handling
  let submitEnabled = true;
  let submitError: {type: 'authentication'} | {type: 'api-error'; message: string} | undefined;

  async function submitForm(): Promise<void> {
    submitEnabled = false;
    validateAll();
    const allFieldsValid = Object.values(fieldErrors).every((error) => error === undefined);
    if (allFieldsValid) {
      console.log(
        location === undefined ? 'Sending new location to API' : 'Updating location via API',
      );
      const url =
        location === undefined ? '/api/v1/locations/' : `/api/v1/locations/${location.id}`;
      const response = await apiPost(url, {
        name,
        countryCode,
        elevation,
        ...(latitude !== null && longitude !== null
          ? {coordinates: {lat: latitude, lon: longitude}}
          : {}),
      });
      switch (response.status) {
        case 201:
        case 204:
          // Success
          addFlash({
            message:
              location === undefined
                ? $i18n.t('location.prose--add-success', {name})
                : $i18n.t('location.prose--update-success', {name}),
            severity: 'success',
            icon: 'fa-circle-check',
          });
          goto('/locations/');
          break;
        case 401:
          submitError = {type: 'authentication'};
          break;
        default: {
          submitError = {
            type: 'api-error',
            message: await extractResponseError(response),
          };
          break;
        }
      }
      submitEnabled = true;
    } else {
      console.warn('Some fields are not valid, not submitting form');
      setTimeout(() => (submitEnabled = true), 200);
    }
  }

  onMount(() => {
    // Reset field errors, so user is not greeted with errors on page load
    resetErrors();
  });
</script>

{#if submitError?.type === 'authentication'}
  <MessageModal
    type="warning"
    title={$i18n.t('common.error--authentication-error')}
    message={$i18n.t('common.error--login-session-expired')}
    showClose={false}
  >
    <section slot="buttons">
      <a
        href="/auth/login/?redirect=/locations/{location == undefined ? '' : `${location.id}/edit`}"
        class="button is-warning">{$i18n.t('navigation.login')}</a
      >
    </section>
  </MessageModal>
{:else if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title={$i18n.t('common.error--api-error')}
    message={location === undefined
      ? $i18n.t('location.error--add-error', {message: submitError.message})
      : $i18n.t('location.error--update-error', {message: submitError.message})}
    showClose={true}
    on:closed={() => (submitError = undefined)}
  />
{/if}

<slot name="title" />

<div class="spaced-headers">
  <form
    method="post"
    on:submit={(event) => {
      event.preventDefault();
      void submitForm();
    }}
  >
    <label class="label" for="name">{$i18n.t('location.title--name')} *</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.name !== undefined}
          type="text"
          id="name"
          name="name"
          bind:value={name}
          required
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-map-marker-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.name !== undefined}
      <div class="field-error">
        {$i18n.t('common.error', {message: fieldErrors.longitude})}
      </div>
    {/if}

    <label class="label" for="country">
      {$i18n.t('location.title--country-code')} *
    </label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.countryCode !== undefined}
          type="text"
          id="country"
          name="country"
          minlength="2"
          maxlength="2"
          bind:value={countryCode}
          required
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-globe-americas"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.countryCode !== undefined}
      <div class="field-error">
        {$i18n.t('common.error', {message: fieldErrors.countryCode})}
      </div>
    {/if}

    <label class="label" for="elevation"
      >{$i18n.t('location.title--elevation')} ({$i18n.t('common.unit--m-asl')}) *</label
    >
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.elevation !== undefined}
          type="number"
          id="elevation"
          name="elevation"
          min="-200"
          max="8000"
          step="1"
          bind:value={elevation}
          required
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-tachometer-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.elevation !== undefined}
      <div class="field-error">
        {$i18n.t('common.error', {message: fieldErrors.elevation})}
      </div>
    {/if}

    <label class="label" for="lat">
      {$i18n.t('location.title--latitude')} (WGS 84)
    </label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.latitude !== undefined}
          type="number"
          id="lat"
          name="lat"
          min="-90"
          max="90"
          step="0.000001"
          bind:this={latitudeInput}
          bind:value={latitude}
          placeholder="47.29553"
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-map-marker-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.latitude !== undefined}
      <div class="field-error">
        {$i18n.t('common.error', {message: fieldErrors.latitude})}
      </div>
    {/if}

    <label class="label" for="lng">
      {$i18n.t('location.title--longitude')} (WGS 84)
    </label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.longitude !== undefined}
          type="number"
          id="lng"
          name="lng"
          min="-180"
          max="180"
          step="0.000001"
          bind:this={longitudeInput}
          bind:value={longitude}
          placeholder="8.91927"
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-map-marker-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.longitude !== undefined}
      <div class="field-error">
        {$i18n.t('common.error', {message: fieldErrors.longitude})}
      </div>
    {/if}

    <div class="map mt-5">
      <SingleMap
        bind:latitude
        bind:longitude
        editable={true}
        center={location?.coordinates}
        zoom={location?.coordinates !== undefined ? 13 : undefined}
        onMarkerChange={handleMarkerChange}
      />
    </div>

    <p>
      <em>
        {$i18n.t('location.prose--hint-double-click')}
      </em>
    </p>

    <div class="content control submitcontrols">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">
        {$i18n.t('common.action--submit')}
      </button>
    </div>

    <p class="content">
      <small><em>* {$i18n.t('common.hint--required-fields')}</em></small>
    </p>
  </form>
</div>

<style>
  .field input.error {
    border: 1px solid #ff3860;
  }

  .field-error {
    color: #ff3860;
    font-size: 0.8em;
    margin-top: -12px;
    margin-bottom: 12px;
  }
</style>
