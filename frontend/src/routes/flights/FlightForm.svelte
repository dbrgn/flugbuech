<script lang="ts">
  import {onMount} from 'svelte';

  import MessageModal from '$lib/components/MessageModal.svelte';
  import {reactive} from '$lib/svelte';

  import {type Flight, type FlightLocation} from './api';
  import type {Glider} from '../gliders/api';

  // Props
  export let flight: Flight | undefined = undefined;
  export let gliders: Glider[];
  export let lastGliderId: number | undefined = undefined;
  export let locations: FlightLocation[];
  export let existingFlightNumbers: number[] = [];

  // Form values
  let files: FileList | undefined = undefined;
  let number: number | null = flight?.number ?? null;
  let glider: number | undefined = lastGliderId; // TODO: Is this validated by the API?
  let launchAt: FlightLocation | undefined = flight?.launchAt;
  let landingAt: FlightLocation | undefined = flight?.landingAt;
  let hikeandfly: boolean = flight?.hikeandfly ?? false;
  let launchDate: string = flight?.launchTime?.toISOString().slice(0, 10) ?? '';
  let launchTime: string = flight?.launchTime?.toISOString().slice(0, 10) ?? '';
  let landingTime: string = flight?.landingTime?.toISOString().slice(0, 10) ?? '';

  // Validation
  const fields = ['number', 'glider', 'launchDate', 'launchTime', 'landingTime'] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    number: undefined,
    glider: undefined,
    launchDate: undefined,
    launchTime: undefined,
    landingTime: undefined,
  };
  function validateNumber(): void {
    fieldErrors = {
      ...fieldErrors,
      number:
        number !== null && existingFlightNumbers.includes(number)
          ? 'Flight number already taken'
          : undefined,
    };
  }
  $: reactive(validateNumber, [number]);
  $: gliderIds = gliders.map((glider) => glider.id).filter((g): g is number => g !== undefined);
  function validateGlider(): void {
    fieldErrors = {
      ...fieldErrors,
      glider: glider !== undefined && !gliderIds.includes(glider) ? 'Unknown glider' : undefined,
    };
  }
  $: reactive(validateGlider, [glider]);
  function validateDatesAndTimes(): void {
    // TODO(#74): Allow dates without time
    fieldErrors = {
      ...fieldErrors,
      launchDate:
        launchTime !== '' && launchDate === ''
          ? 'Date must not be empty if launch time is set'
          : undefined,
      launchTime:
        launchDate !== '' && launchTime === ''
          ? 'Launch time must not be empty if date is set'
          : undefined,
      landingTime:
        launchDate !== '' && landingTime === ''
          ? 'Landing time must not be empty if date is set'
          : undefined,
    };
  }
  $: reactive(validateDatesAndTimes, [launchDate, launchTime, landingTime]);
  function validateAll(): void {
    validateNumber();
    validateGlider();
    validateDatesAndTimes();
  }
  function resetErrors(): void {
    for (const field of fields) {
      fieldErrors[field] = undefined;
    }
  }

  // Flight duration display
  let flightDuration: string | undefined;
  function recalculateDuration(): void {
    if (launchTime !== '' && landingTime !== '') {
      const [launchHour, launchMinute] = launchTime.split(':').map((v) => parseInt(v));
      const [landingHour, landingMinute] = landingTime.split(':').map((v) => parseInt(v));

      const launch = launchHour * 60 + launchMinute;
      const landing = landingHour * 60 + landingMinute;
      let duration = landing - launch;
      if (duration < 0) {
        duration += 1440;
      }
      const hours = Math.floor(duration / 60);
      const minutes = (duration % 60).toString().padStart(2, '0');
      flightDuration = `+${hours}:${minutes}`;
    } else {
      flightDuration = undefined;
    }
  }
  $: reactive(recalculateDuration, [launchTime, landingTime]);

  // Error handling
  let submitEnabled = true;
  let submitError: {type: 'authentication'} | {type: 'api-error'; message: string} | undefined;

  async function submitForm(): Promise<void> {
    submitEnabled = false;
    validateAll();
    if (Object.values(fieldErrors).every((error) => error === undefined)) {
      // All fields valid
      console.log(flight === undefined ? 'Sending new flight to API' : 'Updating flight via API');
      // TODO
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
      <a
        href="/auth/login/?redirect=/flights/{flight == undefined ? '' : `${flight.id}/edit`}"
        class="button is-warning">Login</a
      >
    </section>
  </MessageModal>
{:else if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message="The location could not be {flight === undefined
      ? 'added'
      : 'updated'} due to an error on the server: {submitError.message}"
    showClose={true}
    on:closed={() => (submitError = undefined)}
  />
{/if}

<slot name="title" />

<slot name="intro" />

<div class="spaced-headers">
  <form
    method="post"
    on:submit={(event) => {
      event.preventDefault();
      void submitForm();
    }}
  >
    <h3 class="title is-4">Basic Information</h3>

    <label class="label" for="igc-file">IGC Flight Recording</label>
    {#if flight !== undefined && flight.hasIgc}
      <p class="content">
        <em>IGC file already uploaded. IGC files cannot be changed after the initial upload.</em>
      </p>
    {:else}
      <div class="field">
        <div class="file has-name">
          <label class="file-label">
            <input class="file-input" type="file" name="igc-file" accept=".igc" bind:files />
            <span class="file-cta">
              <span class="file-icon">
                <i class="fa-solid fa-upload"></i>
              </span>
              <span class="file-label"> Click to upload IGC file </span>
            </span>
            <span class="file-name">No file selected…</span>
            <!-- TODO: Update filename on change -->
          </label>
        </div>
      </div>
    {/if}

    <label class="label" for="number">Flight Number</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.number !== undefined}
          type="number"
          min="0"
          step="1"
          name="number"
          bind:value={number}
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-list-ol"></i>
        </div>
        {#if existingFlightNumbers.length > 0}
          <p class="formhint">Highest flight number so far: {Math.max(...existingFlightNumbers)}</p>
        {/if}
      </div>
    </div>
    {#if fieldErrors.number !== undefined}
      <div class="field-error">Error: {fieldErrors.number}</div>
    {/if}

    <label class="label" for="glider">Glider</label>
    <div class="field">
      <div class="control is-expanded has-icons-left">
        <div class="select is-fullwidth">
          <select name="glider" class:error={fieldErrors.glider !== undefined} bind:value={glider}>
            <option value={undefined}></option>
            {#each gliders as glider}
              <option value={glider.id}>
                {glider.manufacturer}
                {glider.model}
              </option>
            {/each}
          </select>
          <div class="icon is-small is-left">
            <i class="fa-solid fa-parachute-box"></i>
          </div>
        </div>
      </div>
    </div>
    {#if fieldErrors.glider !== undefined}
      <div class="field-error">Error: {fieldErrors.glider}</div>
    {/if}

    <h3 class="title is-4">Launch &amp; Landing</h3>

    <div class="columns">
      <div class="column">
        <label class="label" for="launchSite">Launch Site</label>
        <div class="control is-expanded has-icons-left">
          <div class="select is-fullwidth">
            <select name="launch_site" bind:value={launchAt}>
              <option value={undefined}></option>
              {#each locations as location}
                <option value={location}>
                  {location.name} [{location.countryCode}, {location.elevation} m]
                </option>
              {/each}
            </select>
            <div class="icon is-small is-left">
              <i class="fas fa-plane-departure"></i>
            </div>
          </div>
        </div>
        <label class="checkbox">
          <input type="checkbox" name="hikeandfly" bind:checked={hikeandfly} />
          Hike &amp; Fly
        </label>
      </div>

      <div class="column">
        <label class="label" for="landingSite">Landing Site</label>
        <div class="control is-expanded has-icons-left">
          <div class="select is-fullwidth">
            <select name="landing_site" bind:value={landingAt}>
              <option value={undefined}></option>
              {#each locations as location}
                <option value={location}>
                  {location.name} [{location.countryCode}, {location.elevation} m]
                </option>
              {/each}
            </select>
            <div class="icon is-small is-left">
              <i class="fas fa-plane-arrival"></i>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="columns">
      <div class="column">
        <label class="label" for="launchDate">Launch Date</label>
        <div class="field">
          <div class="control has-icons-left">
            <input
              class="input"
              type="date"
              name="launch_date"
              class:error={fieldErrors.launchDate !== undefined}
              bind:value={launchDate}
            />
            <div class="icon is-small is-left">
              <i class="fas fa-calendar-alt"></i>
            </div>
          </div>
        </div>
        {#if fieldErrors.launchDate !== undefined}
          <div class="field-error">Error: {fieldErrors.launchDate}</div>
        {/if}
      </div>
      <div class="column">
        <label class="label" for="launchTime">Launch Time (UTC)</label>
        <div class="field">
          <div class="control has-icons-left">
            <input
              class="input"
              type="time"
              step="60"
              name="launch_time"
              class:error={fieldErrors.launchTime !== undefined}
              bind:value={launchTime}
            />
            <div class="icon is-small is-left">
              <i class="fas fa-clock"></i>
            </div>
          </div>
        </div>
        {#if fieldErrors.launchTime !== undefined}
          <div class="field-error">Error: {fieldErrors.launchTime}</div>
        {/if}
      </div>

      <div class="column">
        <label class="label" for="landingTime">Landing Time (UTC)</label>
        <div class="field has-addons">
          <div class="control is-expanded has-icons-left">
            <input
              class="input"
              type="time"
              step="60"
              name="landing_time"
              bind:value={landingTime}
            />
            <div class="icon is-small is-left">
              <i class="fas fa-clock"></i>
            </div>
          </div>
          <p class="control" class:is-hidden={flightDuration === undefined}>
            <a class="button is-static" href=".">{flightDuration}</a>
          </p>
        </div>
        {#if fieldErrors.landingTime !== undefined}
          <div class="field-error">Error: {fieldErrors.landingTime}</div>
        {/if}
      </div>
    </div>

    <div class="content control submitcontrols">
      <button class="button is-info" disabled={!submitEnabled} type="submit">Submit</button>
    </div>

    <p class="content"><small><em>* Required fields</em></small></p>
  </form>
</div>

<style>
  .field input.error,
  .field select.error {
    border: 1px solid #ff3860;
  }

  .field-error {
    color: #ff3860;
    font-size: 0.8em;
    margin-top: -12px;
    margin-bottom: 12px;
  }
</style>
