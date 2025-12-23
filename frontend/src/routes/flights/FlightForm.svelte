<script lang="ts">
  import {onMount} from 'svelte';

  import {u8aToBase64} from '$lib/base64';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {countryCodeToFlag} from '$lib/formatters';
  import {i18n} from '$lib/i18n';
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
  let files: FileList | undefined;
  let igcBase64: string | undefined;
  let number: number | null =
    flight?.number ??
    (existingFlightNumbers.length > 0 ? Math.max(...existingFlightNumbers) + 1 : null);
  let glider: number | undefined = flight?.gliderId ?? lastGliderId;
  // Note: For the select input value binding to work correctly, entries from `locations` must be
  // used, even if they're compatible with the type included in the flight data.
  let launchAt: FlightLocation | undefined = locations.find(
    (location) => location.id == flight?.launchAt?.id,
  );
  let landingAt: FlightLocation | undefined = locations.find(
    (location) => location.id == flight?.landingAt?.id,
  );
  // Text input values for location autocomplete
  let launchAtText: string = launchAt
    ? `${countryCodeToFlag(launchAt.countryCode)} ${launchAt.name} ${launchAt.elevation} m`
    : '';
  let landingAtText: string = landingAt
    ? `${countryCodeToFlag(landingAt.countryCode)} ${landingAt.name} ${landingAt.elevation} m`
    : '';
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
          ? $i18n.t('flight.error--flight-number-taken')
          : undefined,
    };
  }
  $: reactive(validateNumber, [number]);
  $: gliderIds = gliders.map((g) => g.id).filter((id): id is number => id !== undefined);
  function validateGlider(): void {
    fieldErrors = {
      ...fieldErrors,
      glider:
        glider !== undefined && !gliderIds.includes(glider)
          ? $i18n.t('flight.error--unknown-glider')
          : undefined,
    };
  }
  $: reactive(validateGlider, [glider]);
  function validateDatesAndTimes(): void {
    // TODO(#74): Allow dates without time
    fieldErrors = {
      ...fieldErrors,
      launchDate:
        launchTime !== '' && launchDate === '' ? $i18n.t('flight.error--date-empty') : undefined,
      launchTime:
        launchDate !== '' && launchTime === ''
          ? $i18n.t('flight.error--launch-time-empty')
          : undefined,
      landingTime:
        launchDate !== '' && landingTime === ''
          ? $i18n.t('flight.error--landing-time-empty')
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
        trackDistance === '' || trackDistance.match(distanceRe)
          ? undefined
          : $i18n.t('flight.error--invalid-distance'),
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
          : $i18n.t('flight.error--invalid-distance'),
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

  // Location lookup from text input
  function handleLocationInput(
    text: string,
    setLocation: (loc: FlightLocation | undefined) => void,
  ): void {
    const match = locations.find(
      (loc) => `${countryCodeToFlag(loc.countryCode)} ${loc.name} ${loc.elevation} m` === text,
    );
    setLocation(match);
  }

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
            ? $i18n.t('flight.prose--add-success')
            : $i18n.t('flight.prose--update-success', {numberOrEmpty: flight.number ?? ''}),
        severity: 'success',
        icon: 'fa-circle-check',
      });
      goto('/flights/');
    } catch (error) {
      if (error instanceof SubmitError) {
        submitError = error.data;
      } else {
        submitError = {
          type: 'api-error',
          message: `${$i18n.t('common.error--api-error')}: ${error}`,
        };
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
      alert($i18n.t('flight.error--not-igc', {name: file.name}));
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
          const foundLaunch = locations.find(
            (value) => value.id === igcMetadata.launch?.locationId,
          );
          if (foundLaunch !== undefined) {
            launchAt = foundLaunch;
            launchAtText = `${countryCodeToFlag(foundLaunch.countryCode)} ${foundLaunch.name} ${foundLaunch.elevation} m`;
          }
        }
        if (landingAt === undefined && igcMetadata.landing?.locationId !== undefined) {
          const foundLanding = locations.find(
            (value) => value.id === igcMetadata.landing?.locationId,
          );
          if (foundLanding !== undefined) {
            landingAt = foundLanding;
            landingAtText = `${countryCodeToFlag(foundLanding.countryCode)} ${foundLanding.name} ${foundLanding.elevation} m`;
          }
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
        alert($i18n.t('flight.error--could-not-process-igc', {message: `${e}`}));
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
    title={$i18n.t('common.error--authentication-error')}
    message={$i18n.t('common.error--login-session-expired')}
    showClose={false}
  >
    <section slot="buttons">
      <a
        href="/auth/login/?redirect=/flights/{flight == undefined ? '' : `${flight.id}/edit`}"
        class="button is-warning"
      >
        {$i18n.t('navigation.login')}
      </a>
    </section>
  </MessageModal>
{:else if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title={$i18n.t('common.error--api-error')}
    message={flight === undefined
      ? $i18n.t('flight.error--add-error', {message: submitError.message})
      : $i18n.t('flight.error--update-error', {message: submitError.message})}
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
    <h3 class="title is-4">{$i18n.t('flight.title--basic-information')}</h3>

    <label class="label" for="igcFile">
      {$i18n.t('flight.title--igc-flight-recording')}
    </label>
    {#if flight?.hasIgc}
      <p class="content">
        <em>
          {$i18n.t('flight.prose--igc-already-uploaded')}
        </em>
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
              <span class="file-label">
                {$i18n.t('flight.prose--click-to-upload-igc')}
              </span>
            </span>
            <span class="file-name">
              {files?.[0].name ?? $i18n.t('flight.prose--no-file-selected')}
            </span>
          </label>
        </div>
      </div>
    {/if}

    <label class="label" for="number">
      {$i18n.t('flight.title--flight-number')}
    </label>
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
          <p class="formhint">
            {$i18n.t('flight.prose--hint-highest-flight-number', {
              number: Math.max(...existingFlightNumbers),
            })}
          </p>
        {/if}
      </div>
    </div>
    {#if fieldErrors.number !== undefined}
      <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.number})}</div>
    {/if}

    <label class="label" for="glider">{$i18n.t('flight.title--glider')}</label>
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
      <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.glider})}</div>
    {/if}

    <h3 class="title is-4">{$i18n.t('flight.title--launch-landing')}</h3>

    <div class="columns">
      <div class="column">
        <label class="label" for="launchSite">
          {$i18n.t('flight.title--launch-site')}
        </label>
        <div class="field">
          <div class="control is-expanded has-icons-left">
            <input
              id="launchSite"
              type="text"
              class="input"
              list="launchSiteList"
              placeholder={$i18n.t('flight.title--launch-site')}
              bind:value={launchAtText}
              on:input={() => handleLocationInput(launchAtText, (loc) => (launchAt = loc))}
            />
            <datalist id="launchSiteList">
              {#each locations as location}
                <option
                  value="{countryCodeToFlag(
                    location.countryCode,
                  )} {location.name} {location.elevation} m"
                ></option>
              {/each}
            </datalist>
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
        <label class="label" for="landingSite">{$i18n.t('flight.title--landing-site')}</label>
        <div class="field">
          <div class="control is-expanded has-icons-left">
            <input
              id="landingSite"
              type="text"
              class="input"
              list="landingSiteList"
              placeholder={$i18n.t('flight.title--landing-site')}
              bind:value={landingAtText}
              on:input={() => handleLocationInput(landingAtText, (loc) => (landingAt = loc))}
            />
            <datalist id="landingSiteList">
              {#each locations as location}
                <option
                  value="{countryCodeToFlag(
                    location.countryCode,
                  )} {location.name} {location.elevation} m"
                ></option>
              {/each}
            </datalist>
            <div class="icon is-small is-left">
              <i class="fa-solid fa-plane-arrival"></i>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="columns">
      <div class="column">
        <label class="label" for="launchDate">
          {$i18n.t('flight.title--launch-date')}
        </label>
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
          <div class="field-error">
            {$i18n.t('common.error', {message: fieldErrors.launchDate})}
          </div>
        {/if}
      </div>
      <div class="column">
        <label class="label" for="launchTime">
          {$i18n.t('flight.title--launch-time')} (UTC)
        </label>
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
          <div class="field-error">
            {$i18n.t('common.error', {message: fieldErrors.launchTime})}
          </div>
        {/if}
      </div>

      <div class="column">
        <label class="label" for="landingTime">
          {$i18n.t('flight.title--landing-time')} (UTC)
        </label>
        <div class="field has-addons">
          <div class="control is-expanded has-icons-left">
            <input id="landingTime" type="time" class="input" step="60" bind:value={landingTime} />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-clock"></i>
            </div>
          </div>
          <p class="control" class:is-hidden={flightDuration === undefined}>
            <a class="button is-static" href="." tabindex="-1">{flightDuration}</a>
          </p>
        </div>
        {#if fieldErrors.landingTime !== undefined}
          <div class="field-error">
            {$i18n.t('common.error', {message: fieldErrors.landingTime})}
          </div>
        {/if}
      </div>
    </div>

    <h3 class="title is-4">
      {$i18n.t('flight.title--gps-track')}
    </h3>

    <label class="label" for="trackDistance">
      {$i18n.t('flight.title--gps-track-distance')}
    </label>
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
        <a class="button is-static" href="." tabindex="-1">km</a>
      </p>
    </div>
    {#if fieldErrors.trackDistance !== undefined}
      <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.trackDistance})}</div>
    {/if}

    <h3 class="title is-4">{$i18n.t('flight.title--xcontest')}</h3>

    <div class="columns">
      <div class="column">
        <label class="label" for="xcontestTracktype">
          {$i18n.t('flight.title--xcontest-track-type')}
        </label>
        <div class="field">
          <div class="control is-expanded has-icons-left">
            <div class="select is-fullwidth">
              <select id="xcontestTracktype" bind:value={xcontestTracktype}>
                <option value={undefined}></option>
                <option value="free_flight">
                  {$i18n.t('common.xcontest--free-flight')}
                </option>
                <option value="flat_triangle">
                  {$i18n.t('common.xcontest--flat-triangle')}
                </option>
                <option value="fai_triangle">
                  {$i18n.t('common.xcontest--fai-triangle')}
                </option>
              </select>
              <div class="icon is-small is-left">
                <i class="fa-solid fa-globe-americas"></i>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="column">
        <label class="label" for="xcontestDistance">
          {$i18n.t('flight.title--xcontest-scored-distance')}
        </label>
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
            <a class="button is-static" href="." tabindex="-1">km</a>
          </p>
        </div>
        {#if fieldErrors.xcontestDistance !== undefined}
          <div class="field-error">
            {$i18n.t('common.error', {message: fieldErrors.xcontestDistance})}
          </div>
        {/if}
      </div>
    </div>

    <label class="label" for="xcontestUrl">
      {$i18n.t('flight.title--xcontest-url')}
    </label>
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

    <h3 class="title is-4">
      {$i18n.t('flight.title--other')}
    </h3>

    <label class="label" for="comment">
      {$i18n.t('flight.title--comment')}
    </label>
    <div class="field">
      <div class="control">
        <textarea
          class="textarea"
          id="comment"
          placeholder={$i18n.t('flight.prose--describe-flight')}
          bind:value={comment}
        ></textarea>
      </div>
    </div>

    <label class="label" for="videoUrl">
      {$i18n.t('flight.title--video-url')}
    </label>
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
      <button class="button is-primary" disabled={!submitEnabled} type="submit">
        {$i18n.t('common.action--submit')}
      </button>
    </div>
  </form>
</div>

<div class="drag-file-overlay" class:is-hidden={!dragFileOverlayVisible}>
  <div>{$i18n.t('flight.prose--drop-file-to-process')}</div>
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
