<script lang="ts">
  import {onMount} from 'svelte';

  import MessageModal from '$lib/components/MessageModal.svelte';
  import {reactive} from '$lib/svelte';

  import {type Flight} from './api';
  import type {Glider} from '../gliders/api';

  // Props
  export let flight: Flight | undefined = undefined;
  export let gliders: Glider[];
  export let lastGliderId: number | undefined = undefined;
  export let existingFlightNumbers: number[] = [];

  // Form values
  let files: FileList | undefined = undefined;
  let number: number | null = flight?.number ?? null;
  let glider: number | undefined = lastGliderId; // TODO: Is this validated by the API?

  // Validation
  const fields = ['number', 'glider'] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    number: undefined,
    glider: undefined,
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
  function validateAll(): void {
    validateNumber();
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
