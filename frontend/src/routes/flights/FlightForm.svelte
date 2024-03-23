<script lang="ts">
  import {onMount} from 'svelte';

  import {u8aToBase64} from '$lib/base64';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {addFlash} from '$lib/stores';
  import {reactive} from '$lib/svelte';
  import type {XContestTracktype} from '$lib/xcontest';

  import {goto} from '$app/navigation';

  import type {Glider} from '../gliders/api';

  import {
    processIgc,
    type Flight,
    type FlightLocation,
    addApiFlight,
    type SubmitErrorData,
    SubmitError,
    type NewApiFlight,
    editApiFlight,
  } from './api';

  // Props
  export let flight: Flight | undefined = undefined;
  export let gliders: Glider[];
  export let lastGliderId: number | undefined = undefined;
  export let locations: FlightLocation[];
  export let existingFlightNumbers: number[] = [];

  // Form values
  // Note: Values for number inputs must allow null!
  let files: FileList | undefined = undefined;
  let igcBase64: string | undefined = undefined;
  let number: number | null = flight?.number ?? null;
  let glider: number | undefined = flight?.gliderId ?? lastGliderId;
  // Note: For the select input value binding to work correctly, entries from `locations` must be
  // used, even if they're compatible with the type included in the flight data.
  let launchAt: FlightLocation | undefined = locations.find(
    (location) => location.id == flight?.launchAt?.id,
  );
  let landingAt: FlightLocation | undefined = locations.find(
    (location) => location.id == flight?.landingAt?.id,
  );
  let hikeandfly: boolean = flight?.hikeandfly ?? false;
  let launchDate: string = flight?.launchTime?.toISOString().slice(0, 10) ?? '';
  let launchTime: string = flight?.launchTime?.toISOString().slice(11, 19) ?? '';
  let landingTime: string = flight?.landingTime?.toISOString().slice(11, 19) ?? '';
  let trackDistance: string = flight?.trackDistance?.toFixed(2) ?? '';
  let xcontestTracktype: XContestTracktype | undefined = flight?.xcontestTracktype;
  let xcontestDistance: string = flight?.xcontestDistance?.toFixed(2) ?? '';
  let xcontestUrl: string = flight?.xcontestUrl ?? '';
  let comment: string = flight?.comment ?? '';
  let videoUrl: string = flight?.videoUrl ?? '';

  // Validation
  const fields = [
    'number',
    'glider',
    'launchDate',
    'launchTime',
    'landingTime',
    'trackDistance',
    'xcontestDistance',
  ] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    number: undefined,
    glider: undefined,
    launchDate: undefined,
    launchTime: undefined,
    landingTime: undefined,
    trackDistance: undefined,
    xcontestDistance: undefined,
  };
  function validateNumber(): void {
    fieldErrors = {
      ...fieldErrors,
      number:
        number !== null && number !== flight?.number && existingFlightNumbers.includes(number)
          ? 'Flight number already taken'
          : undefined,
    };
  }
  $: reactive(validateNumber, [number]);
  $: gliderIds = gliders.map((g) => g.id).filter((id): id is number => id !== undefined);
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
  function validateTrackDistance(): void {
    // TODO: Create proper NumberInput component
    const distanceRe = /^[0-9]+(\.[0-9]+)?$/u;
    trackDistance = trackDistance.replace(',', '.');
    fieldErrors = {
      ...fieldErrors,
      trackDistance:
        trackDistance === '' || trackDistance.match(distanceRe) ? undefined : 'Invalid distance',
    };
  }
  $: reactive(validateTrackDistance, [trackDistance]);
  function validateXContestDistance(): void {
    // TODO: Create proper NumberInput component
    const distanceRe = /^[0-9]+(\.[0-9]+)?$/u;
    xcontestDistance = xcontestDistance.replace(',', '.');
    fieldErrors = {
      ...fieldErrors,
      xcontestDistance:
        xcontestDistance === '' || xcontestDistance.match(distanceRe)
          ? undefined
          : 'Invalid distance',
    };
  }
  $: reactive(validateXContestDistance, [xcontestDistance]);
  function validateAll(): void {
    validateNumber();
    validateGlider();
    validateDatesAndTimes();
    validateTrackDistance();
    validateXContestDistance();
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
  let submitError: SubmitErrorData | undefined;

  // Form submission
  async function submitForm(): Promise<void> {
    submitEnabled = false;

    // Validate
    validateAll();
    const allFieldsValid = Object.values(fieldErrors).every((error) => error === undefined);
    if (!allFieldsValid) {
      console.warn('Some fields are not valid, not submitting form');
      setTimeout(() => (submitEnabled = true), 200);
      return;
    }

    console.log(flight === undefined ? 'Sending new flight to API' : 'Updating flight via API');
    const flightData: NewApiFlight = {
      number: number ?? undefined,
      glider,
      launchSite: launchAt?.id,
      landingSite: landingAt?.id,
      launchDate: launchDate === '' ? undefined : launchDate,
      launchTime: launchTime === '' ? undefined : launchTime,
      landingTime: landingTime === '' ? undefined : landingTime,
      hikeandfly,
      trackDistance: trackDistance === '' ? undefined : parseFloat(trackDistance),
      xcontestTracktype,
      xcontestDistance: xcontestDistance === '' ? undefined : parseFloat(xcontestDistance),
      xcontestUrl: xcontestUrl === '' ? undefined : xcontestUrl,
      comment,
      videoUrl: videoUrl === '' ? undefined : videoUrl,
      igcData: igcBase64,
    };
    try {
      if (flight === undefined) {
        await addApiFlight(flightData);
      } else {
        await editApiFlight(flight.id, flightData);
      }
      addFlash({
        message:
          flight === undefined
            ? 'Flight successfully added'
            : `Flight ${flight.number ?? ''} successfully updated`,
        severity: 'success',
        icon: 'fa-circle-check',
      });
      goto('/flights/');
    } catch (error) {
      if (error instanceof SubmitError) {
        submitError = error.data;
      } else {
        submitError = {type: 'api-error', message: `Unknown API error: ${error}`};
      }
    }
    submitEnabled = true;
  }

  // TODO: Unit test
  function hmsToTimevalue(hms: [number, number, number]) {
    let hours = hms[0];
    let minutes = Math.round(hms[1] + hms[2] / 100);
    if (minutes === 60) {
      minutes = 0;
      hours += 1;
    }
    const h = hours.toString().padStart(2, '0');
    const m = minutes.toString().padStart(2, '0');
    return `${h}:${m}`;
  }

  // Handle file uploads
  function onFileInputChange(): void {
    // Ensure that file is present
    const file = files?.[0];
    if (file === undefined) {
      return;
    }

    // Only process IGC files
    if (!file.name.toLocaleLowerCase().endsWith('.igc')) {
      alert(`File "${file.name}" does not end with .igc, ignoring`);
      files = undefined;
      return;
    }

    console.log('Submitting IGC data');
    processIgc(file)
      .then((igcMetadata) => {
        // Determine flight date
        if (igcMetadata.dateYmd) {
          if (launchDate === '') {
            const y = igcMetadata.dateYmd[0].toString();
            const m = igcMetadata.dateYmd[1].toString().padStart(2, '0');
            const d = igcMetadata.dateYmd[2].toString().padStart(2, '0');
            launchDate = `${y}-${m}-${d}`;
          }
        }

        // Determine launch and landing time
        if (launchTime === '' && igcMetadata.launch?.timeHms !== undefined) {
          launchTime = hmsToTimevalue(igcMetadata.launch.timeHms);
        }
        if (landingTime === '' && igcMetadata.landing?.timeHms !== undefined) {
          landingTime = hmsToTimevalue(igcMetadata.landing.timeHms);
        }

        // Determine launch and landing sites
        if (launchAt === undefined && igcMetadata.launch?.locationId !== undefined) {
          launchAt = locations.find((value) => value.id === igcMetadata.launch?.locationId);
        }
        if (landingAt === undefined && igcMetadata.landing?.locationId !== undefined) {
          landingAt = locations.find((value) => value.id === igcMetadata.landing?.locationId);
        }

        // Determine track distance
        if (trackDistance === '') {
          trackDistance = igcMetadata.trackDistance.toFixed(2);
        }

        // Because multipart form submissions suck, we convert the file to base64 and later submit
        // it as text. Not nice either, but at least allows us to use regular form parsing. And
        // because we want to store the file in the database, we can't stream it to disk anyways.
        const reader = new FileReader();
        reader.onload = (e) => {
          const bytes = e.target?.result;
          if (bytes instanceof ArrayBuffer) {
            igcBase64 = u8aToBase64(new Uint8Array(bytes), {urlSafe: true, noPad: true});
          } else {
            throw new Error('Unexpected file read format');
          }
        };
        reader.readAsArrayBuffer(file);
      })
      .catch((e) => {
        alert(`Could not process IGC file: ${e}`);
      });
  }
  $: reactive(onFileInputChange, [files]);

  // File drop target
  let dragFileOverlayVisible = false;
  function setUpDropTarget(): void {
    function onDragOver(e: DragEvent) {
      e.stopPropagation();
      e.preventDefault();
      dragFileOverlayVisible = true;
    }

    function onDragLeave(e: DragEvent) {
      e.stopPropagation();
      e.preventDefault();
      dragFileOverlayVisible = false;
    }

    function onDrop(e: DragEvent) {
      e.stopPropagation();
      e.preventDefault();
      onDragLeave(e);
      if (e.dataTransfer && e.dataTransfer.files) {
        files = e.dataTransfer.files;
      }
    }

    const body = document.querySelector('body');
    if (body !== null) {
      body.addEventListener('dragover', onDragOver);
      body.addEventListener('dragenter', onDragOver);
      body.addEventListener('dragleave', onDragLeave);
      body.addEventListener('dragend', onDragLeave);
      body.addEventListener('drop', onDrop);
    }
  }

  onMount(() => {
    // Reset field errors, so user is not greeted with errors on page load
    resetErrors();

    // Set up drag & drop handling for flights that don't yet have an IGC file
    if (flight === undefined || !flight.hasIgc) {
      setUpDropTarget();
    }
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

    <label class="label" for="igcFile">IGC Flight Recording</label>
    {#if flight?.hasIgc}
      <p class="content">
        <em>IGC file already uploaded. IGC files cannot be changed after the initial upload.</em>
      </p>
    {:else}
      <div class="field">
        <div class="file has-name">
          <label class="file-label">
            <input id="igcFile" type="file" class="file-input" accept=".igc" bind:files />
            <span class="file-cta">
              <span class="file-icon">
                <i class="fa-solid fa-upload"></i>
              </span>
              <span class="file-label"> Click to upload IGC file </span>
            </span>
            <span class="file-name">{files?.[0].name ?? 'No file selected…'}</span>
          </label>
        </div>
      </div>
    {/if}

    <label class="label" for="number">Flight Number</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          id="number"
          type="number"
          class="input"
          class:error={fieldErrors.number !== undefined}
          step="1"
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
          <select id="glider" class:error={fieldErrors.glider !== undefined} bind:value={glider}>
            <option value={undefined}></option>
            {#each gliders as g}
              <option value={g.id}>
                {g.manufacturer}
                {g.model}
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
            <select id="launchSite" bind:value={launchAt}>
              <option value={undefined}></option>
              {#each locations as location}
                <option value={location}>
                  {location.name} [{location.countryCode}, {location.elevation} m]
                </option>
              {/each}
            </select>
            <div class="icon is-small is-left">
              <i class="fa-solid fa-plane-departure"></i>
            </div>
          </div>
        </div>
        <label class="checkbox">
          <input type="checkbox" id="hikeandfly" bind:checked={hikeandfly} />
          Hike &amp; Fly
        </label>
      </div>

      <div class="column">
        <label class="label" for="landingSite">Landing Site</label>
        <div class="control is-expanded has-icons-left">
          <div class="select is-fullwidth">
            <select id="landingSite" bind:value={landingAt}>
              <option value={undefined}></option>
              {#each locations as location}
                <option value={location}>
                  {location.name} [{location.countryCode}, {location.elevation} m]
                </option>
              {/each}
            </select>
            <div class="icon is-small is-left">
              <i class="fa-solid fa-plane-arrival"></i>
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
              id="launchDate"
              type="date"
              class="input"
              class:error={fieldErrors.launchDate !== undefined}
              bind:value={launchDate}
            />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-calendar-alt"></i>
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
              id="launchTime"
              type="time"
              class="input"
              step="60"
              class:error={fieldErrors.launchTime !== undefined}
              bind:value={launchTime}
            />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-clock"></i>
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
            <input id="landingTime" type="time" class="input" step="60" bind:value={landingTime} />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-clock"></i>
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

    <h3 class="title is-4">GPS Track</h3>

    <label class="label" for="trackDistance">GPS Track Distance</label>
    <div class="field has-addons">
      <div class="control is-expanded has-icons-left">
        <input
          id="trackDistance"
          type="text"
          class="input"
          class:error={fieldErrors.trackDistance !== undefined}
          bind:value={trackDistance}
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-ruler"></i>
        </div>
      </div>
      <p class="control">
        <a class="button is-static" href=".">km</a>
      </p>
    </div>
    {#if fieldErrors.trackDistance !== undefined}
      <div class="field-error">Error: {fieldErrors.trackDistance}</div>
    {/if}

    <h3 class="title is-4">XContest</h3>

    <div class="columns">
      <div class="column">
        <label class="label" for="xcontestTracktype">XContest Track Type</label>
        <div class="field">
          <div class="control is-expanded has-icons-left">
            <div class="select is-fullwidth">
              <select id="xcontestTracktype" bind:value={xcontestTracktype}>
                <option value={undefined}></option>
                <option value="free_flight">Free Flight</option>
                <option value="flat_triangle">Flat Triangle</option>
                <option value="fai_triangle">FAI Triangle</option>
              </select>
              <div class="icon is-small is-left">
                <i class="fa-solid fa-globe-americas"></i>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="column">
        <label class="label" for="xcontestDistance">XContest Scored Distance</label>
        <div class="field has-addons">
          <div class="control is-expanded has-icons-left">
            <input
              id="xcontestDistance"
              type="text"
              class="input"
              class:error={fieldErrors.xcontestDistance !== undefined}
              bind:value={xcontestDistance}
            />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-ruler"></i>
            </div>
          </div>
          <p class="control">
            <a class="button is-static" href=".">km</a>
          </p>
        </div>
        {#if fieldErrors.xcontestDistance !== undefined}
          <div class="field-error">Error: {fieldErrors.xcontestDistance}</div>
        {/if}
      </div>
    </div>

    <label class="label" for="xcontestUrl">XContest URL</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          type="url"
          pattern="https?://.*"
          id="xcontestUrl"
          placeholder="https://www.xcontest.org/..."
          bind:value={xcontestUrl}
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-link"></i>
        </div>
      </div>
    </div>

    <h3 class="title is-4">Other</h3>

    <label class="label" for="comment">Comment</label>
    <div class="field">
      <div class="control">
        <textarea
          class="textarea"
          id="comment"
          placeholder="Describe your flight"
          bind:value={comment}
        ></textarea>
      </div>
    </div>

    <label class="label" for="videoUrl">Video URL</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          type="url"
          pattern="https?://.*"
          id="videoUrl"
          placeholder="https://www.youtube.com/..."
          bind:value={videoUrl}
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-film"></i>
        </div>
      </div>
    </div>

    <div class="content control submitcontrols">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">Submit</button>
    </div>
  </form>
</div>

<div class="drag-file-overlay" class:is-hidden={!dragFileOverlayVisible}>
  <div>Drop file to process</div>
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

  .drag-file-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9999;
    background-color: rgba(0, 0, 0, 0.7);
  }

  .drag-file-overlay div {
    text-align: center;
    color: white;
    font-size: 4em;
    position: relative;
    padding-top: 0;
    top: 50%;
    transform: translatey(-50%);
  }
</style>
