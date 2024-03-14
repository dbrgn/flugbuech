<script lang="ts">
  import {onMount} from 'svelte';

  import {apiPost, extractResponseError} from '$lib/api';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {addFlash} from '$lib/stores';
  import {reactive} from '$lib/svelte';

  import {goto} from '$app/navigation';

  import type {Glider} from './api';

  // Props
  export let glider: Glider | undefined = undefined;

  // Form values
  let manufacturer: string = glider?.manufacturer ?? '';
  let model: string = glider?.model ?? '';
  let since: string = glider?.since ?? '';
  let until: string = glider?.until ?? '';
  let source: string = glider?.source ?? '';
  let cost: number | null = glider?.cost ?? null;
  let comment: string = glider?.comment ?? '';

  // Validation
  const fields = ['manufacturer', 'model', 'since', 'until', 'source', 'cost', 'comment'] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    manufacturer: undefined,
    model: undefined,
    since: undefined,
    until: undefined,
    source: undefined,
    cost: undefined,
    comment: undefined,
  };
  function validateManufacturer(): void {
    fieldErrors = {
      ...fieldErrors,
      manufacturer: manufacturer.length < 1 ? 'Manufacturer must not be empty' : undefined,
    };
  }
  $: reactive(validateManufacturer, [manufacturer]);
  function validateModel(): void {
    fieldErrors = {
      ...fieldErrors,
      model: model.length < 1 ? 'Model must not be empty' : undefined,
    };
  }
  $: reactive(validateModel, [model]);
  function validateUntil(): void {
    fieldErrors = {
      ...fieldErrors,
      until:
        since !== null && until !== null && new Date(since) > new Date(until)
          ? '"Until" must not be earlier than "Since"'
          : undefined,
    };
  }
  $: reactive(validateUntil, [since, until]);
  function validateCost(): void {
    fieldErrors = {
      ...fieldErrors,
      cost: cost !== null && cost < 0 ? 'Cost must not be negative' : undefined,
    };
  }
  $: reactive(validateCost, [cost]);
  function validateAll(): void {
    validateManufacturer();
    validateModel();
    validateCost();
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
      console.log(glider === undefined ? 'Sending new glider to API' : 'Updating glider via API');
      const url = glider === undefined ? '/api/v1/gliders/' : `/api/v1/gliders/${glider.id}`;
      const response = await apiPost(url, {
        manufacturer,
        model,
        since,
        until,
        source,
        cost,
        comment,
      });
      switch (response.status) {
        case 201:
        case 204:
          // Success
          addFlash({
            message: `Glider "${manufacturer} ${model}" successfully ${
              glider === undefined ? 'added' : 'updated'
            }`,
            severity: 'success',
            icon: 'fa-circle-check',
          });
          goto('/gliders/');
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
      <a
        href="/auth/login/?redirect=/gliders/{glider == undefined ? '' : `${glider.id}/edit`}"
        class="button is-warning">Login</a
      >
    </section>
  </MessageModal>
{:else if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message="The glider could not be {glider === undefined
      ? 'added'
      : 'updated'} due to an error on the server: {submitError.message}"
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
    <label class="label" for="manufacturer">Manufacturer *</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.manufacturer !== undefined}
          type="text"
          id="manufacturer"
          name="manufacturer"
          bind:value={manufacturer}
          required
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-industry"></i>
        </div>
        <p class="formhint">The glider manufacturer, e.g. "Advance"</p>
      </div>
    </div>
    {#if fieldErrors.manufacturer !== undefined}
      <div class="field-error">Error: {fieldErrors.manufacturer}</div>
    {/if}

    <label class="label" for="model">Model *</label>
    <div class="field">
      <div class="control has-icons-left">
        <input
          class="input"
          class:error={fieldErrors.model !== undefined}
          type="text"
          id="model"
          name="model"
          bind:value={model}
          required
        />
        <div class="icon is-small is-left">
          <i class="fa-solid fa-parachute-box"></i>
        </div>
        <p class="formhint">The glider model, e.g. "Epsilon 8"</p>
      </div>
    </div>
    {#if fieldErrors.model !== undefined}
      <div class="field-error">Error: {fieldErrors.model}</div>
    {/if}

    <div class="columns">
      <div class="column">
        <label class="label" for="since">Since</label>
        <div class="field">
          <div class="control has-icons-left">
            <input
              class="input"
              class:error={fieldErrors.since !== undefined}
              type="date"
              id="since"
              name="since"
              bind:value={since}
            />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-calendar-alt"></i>
            </div>
            <p class="formhint">When did you acquire this glider?</p>
          </div>
        </div>
        {#if fieldErrors.since !== undefined}
          <div class="field-error in-column">Error: {fieldErrors.since}</div>
        {/if}
      </div>

      <div class="column">
        <label class="label" for="until">Until</label>
        <div class="field">
          <div class="control has-icons-left">
            <input
              class="input"
              class:error={fieldErrors.until !== undefined}
              type="date"
              id="until"
              name="until"
              bind:value={until}
            />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-calendar-alt"></i>
            </div>
            <p class="formhint">
              Until when did you own this glider? (Leave empty if you still have it)
            </p>
          </div>
        </div>
        {#if fieldErrors.until !== undefined}
          <div class="field-error in-column">Error: {fieldErrors.until}</div>
        {/if}
      </div>
    </div>

    <div class="columns">
      <div class="column">
        <label class="label" for="source">Source</label>
        <div class="field">
          <div class="control has-icons-left">
            <input class="input" type="text" id="source" name="source" bind:value={source} />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-shopping-cart"></i>
            </div>
            <p class="formhint">Where did you get this glider from? (e.g. "Flybubble Shop")</p>
          </div>
        </div>
        {#if fieldErrors.source !== undefined}
          <div class="field-error in-column">Error: {fieldErrors.source}</div>
        {/if}
      </div>

      <div class="column">
        <label class="label" for="cost">Cost</label>
        <div class="field">
          <div class="control has-icons-left">
            <input
              class="input"
              type="number"
              min="0"
              step="1"
              id="cost"
              name="cost"
              bind:value={cost}
            />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-euro-sign"></i>
            </div>
            <p class="formhint">How much did you pay for this glider, in your own currency?</p>
          </div>
        </div>
        {#if fieldErrors.cost !== undefined}
          <div class="field-error in-column">Error: {fieldErrors.cost}</div>
        {/if}
      </div>
    </div>

    <label class="label" for="comment">Comment</label>
    <div class="field">
      <div class="control">
        <textarea class="textarea" id="comment" name="comment" bind:value={comment} />
      </div>
    </div>
    {#if fieldErrors.comment !== undefined}
      <div class="field-error">Error: {fieldErrors.comment}</div>
    {/if}

    <div class="content control submitcontrols">
      <button class="button is-info" disabled={!submitEnabled} type="submit">Submit</button>
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
  }

  .field-error:not(.in-column) {
    margin-bottom: 12px;
  }
</style>
