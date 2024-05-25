<script lang="ts">
  import {onMount} from 'svelte';

  import {apiPost, extractResponseError} from '$lib/api';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {i18n} from '$lib/i18n';
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
      manufacturer:
        manufacturer.length < 1 ? $i18n.t('glider.warning--manufacturer-empty') : undefined,
    };
  }
  $: reactive(validateManufacturer, [manufacturer]);
  function validateModel(): void {
    fieldErrors = {
      ...fieldErrors,
      model: model.length < 1 ? $i18n.t('glider.warning--model-empty') : undefined,
    };
  }
  $: reactive(validateModel, [model]);
  function validateUntil(): void {
    fieldErrors = {
      ...fieldErrors,
      until:
        since !== null && until !== null && new Date(since) > new Date(until)
          ? $i18n.t('glider.warning--until-earlier-than-since')
          : undefined,
    };
  }
  $: reactive(validateUntil, [since, until]);
  function validateCost(): void {
    fieldErrors = {
      ...fieldErrors,
      cost: cost !== null && cost < 0 ? $i18n.t('glider.warning--cost-negative') : undefined,
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
            message:
              glider === undefined
                ? $i18n.t('glider.prose--add-success', {manufacturer, model})
                : $i18n.t('glider.prose--update-success', {manufacturer, model}),
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
    title={$i18n.t('common.error--authentication-error')}
    message={$i18n.t('common.error--login-session-expired')}
    showClose={false}
  >
    <section slot="buttons">
      <a
        href="/auth/login/?redirect=/gliders/{glider == undefined ? '' : `${glider.id}/edit`}"
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
    message={glider === undefined
      ? $i18n.t('glider.error--add-error', {message: submitError.message})
      : $i18n.t('glider.error--update-error', {message: submitError.message})}
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
    <label class="label" for="manufacturer">
      {$i18n.t('glider.title--manufacturer')} *
    </label>
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
        <p class="formhint">
          {$i18n.t('glider.hint--manufacturer')}
        </p>
      </div>
    </div>
    {#if fieldErrors.manufacturer !== undefined}
      <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.manufacturer})}</div>
    {/if}

    <label class="label" for="model">
      {$i18n.t('glider.title--model')} *
    </label>
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
        <p class="formhint">
          {$i18n.t('glider.hint--model')}
        </p>
      </div>
    </div>
    {#if fieldErrors.model !== undefined}
      <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.model})}</div>
    {/if}

    <div class="columns">
      <div class="column">
        <label class="label" for="since">{$i18n.t('glider.title--since')}</label>
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
            <p class="formhint">
              {$i18n.t('glider.hint--since')}
            </p>
          </div>
        </div>
        {#if fieldErrors.since !== undefined}
          <div class="field-error in-column">
            {$i18n.t('common.error', {message: fieldErrors.since})}
          </div>
        {/if}
      </div>

      <div class="column">
        <label class="label" for="until">{$i18n.t('glider.title--until')}</label>
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
              {$i18n.t('glider.hint--until')}
            </p>
          </div>
        </div>
        {#if fieldErrors.until !== undefined}
          <div class="field-error in-column">
            {$i18n.t('common.error', {message: fieldErrors.until})}
          </div>
        {/if}
      </div>
    </div>

    <div class="columns">
      <div class="column">
        <label class="label" for="source">{$i18n.t('glider.title--source')}</label>
        <div class="field">
          <div class="control has-icons-left">
            <input class="input" type="text" id="source" name="source" bind:value={source} />
            <div class="icon is-small is-left">
              <i class="fa-solid fa-shopping-cart"></i>
            </div>
            <p class="formhint">
              {$i18n.t('glider.hint--source')}
            </p>
          </div>
        </div>
        {#if fieldErrors.source !== undefined}
          <div class="field-error in-column">
            {$i18n.t('common.error', {message: fieldErrors.source})}
          </div>
        {/if}
      </div>

      <div class="column">
        <label class="label" for="cost">{$i18n.t('glider.title--cost')}</label>
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
            <p class="formhint">
              {$i18n.t('glider.hint--cost')}
            </p>
          </div>
        </div>
        {#if fieldErrors.cost !== undefined}
          <div class="field-error in-column">
            {$i18n.t('common.error', {message: fieldErrors.cost})}
          </div>
        {/if}
      </div>
    </div>

    <label class="label" for="comment">{$i18n.t('glider.title--comment')}</label>
    <div class="field">
      <div class="control">
        <textarea class="textarea" id="comment" name="comment" bind:value={comment} />
      </div>
    </div>
    {#if fieldErrors.comment !== undefined}
      <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.comment})}</div>
    {/if}

    <div class="content control submitcontrols">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">
        {$i18n.t('common.action--submit')}
      </button>
    </div>

    <p class="content"><small><em>* {$i18n.t('common.hint--required-fields')}</em></small></p>
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
