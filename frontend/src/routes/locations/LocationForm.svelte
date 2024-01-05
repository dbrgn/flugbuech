<script lang="ts">
  import {onMount} from 'svelte';

  import {goto} from '$app/navigation';

  import MapComponent from '$lib/components/Map.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {addFlash} from '$lib/stores';
  import {reactive} from '$lib/svelte';

  import {type Location} from './api';
  import {apiPost, extractResponseError} from '$lib/api';

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
      name: name.length < 1 ? 'Name must not be empty' : undefined,
    };
  }
  $: reactive(validateName, [name]);
  function validateCountryCode(): void {
    fieldErrors = {
      ...fieldErrors,
      countryCode:
        countryCode.match(/^[A-Z]{2}$/u) === null ? 'Country code must have 2 letters' : undefined,
    };
  }
  $: reactive(validateCountryCode, [countryCode]);
  function validateElevation(): void {
    fieldErrors = {
      ...fieldErrors,
      elevation: elevation === null ? 'Elevation must not be empty' : undefined,
    };
  }
  $: reactive(validateElevation, [elevation]);
  function validateLatitude(): void {
    fieldErrors = {
      ...fieldErrors,
      latitude:
        longitude !== null && latitude === null
          ? 'If longitude is set, latitude must not be empty'
          : undefined,
    };
  }
  $: reactive(validateLatitude, [latitude, longitude]);
  function validateLongitude(): void {
    fieldErrors = {
      ...fieldErrors,
      longitude:
        latitude !== null && longitude === null
          ? 'If latitude is set, longitude must not be empty'
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
    if (Object.values(fieldErrors).every((error) => error === undefined)) {
      // All fields valid
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
                ? `Location successfully added`
                : `Location "${name}" successfully updated`,
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
    title="Authentication Error"
    message="Your login session has expired. Please log in again."
    showClose={false}
  >
    <section slot="buttons">
      <a href="/auth/login/" class="button is-warning">Login</a>
    </section>
  </MessageModal>
{:else if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message="The location could not be {location === undefined
      ? 'added'
      : 'updated'} due to an error on the server: {submitError.message}"
    showClose={true}
    on:closed={() => (submitError = undefined)}
  />
{/if}

<slot name="title" />

<div class="spaced-headers">
  <form on:submit={() => void submitForm()}>
    <label class="label" for="name">Name *</label>
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
          <i class="fas fa-map-marker-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.name !== undefined}
      <div class="field-error">Error: {fieldErrors.name}</div>
    {/if}

    <label class="label" for="country">Country Code (2 Letters) *</label>
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
          <i class="fas fa-globe-americas"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.countryCode !== undefined}
      <div class="field-error">Error: {fieldErrors.countryCode}</div>
    {/if}

    <label class="label" for="elevation">Elevation (m ASL) *</label>
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
          <i class="fas fa-tachometer-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.elevation !== undefined}
      <div class="field-error">Error: {fieldErrors.elevation}</div>
    {/if}

    <label class="label" for="lat">Latitude</label>
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
          <i class="fas fa-map-marker-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.latitude !== undefined}
      <div class="field-error">Error: {fieldErrors.latitude}</div>
    {/if}

    <label class="label" for="lng">Longitude</label>
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
          <i class="fas fa-map-marker-alt"></i>
        </div>
      </div>
    </div>
    {#if fieldErrors.longitude !== undefined}
      <div class="field-error">Error: {fieldErrors.longitude}</div>
    {/if}

    <MapComponent
      latInput={latitudeInput}
      lngInput={longitudeInput}
      position={location?.coordinates}
      zoom={location?.coordinates !== undefined ? 13 : undefined}
    />

    <p><em>Note: Double-click on the map to update the location coordinates.</em></p>

    <div class="content control submitcontrols">
      <button class="button is-info" disabled={!submitEnabled} on:click={submitForm}>Submit</button>
    </div>

    <p class="content"><small><em>* Required fields</em></small></p>
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
