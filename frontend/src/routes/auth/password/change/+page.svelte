<script lang="ts">
  import {onMount} from 'svelte';

  import {ensureError} from '$lib/assert';
  import {requireLogin} from '$lib/auth';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import {MIN_PASSWORD_LENGTH} from '$lib/constants';
  import {i18n} from '$lib/i18n';
  import {addFlash, loginState} from '$lib/stores';
  import {reactive} from '$lib/svelte';

  import {goto} from '$app/navigation';

  import PasswordFormInputField from './PasswordFormInputField.svelte';
  import {apiChangePassword} from './api';

  // Form values
  let current: string = '';
  let new1: string = '';
  let new2: string = '';

  // Element bindings
  let flashes: Flashes;

  // Validation
  const fields = ['current', 'new1', 'new2'] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    current: undefined,
    new1: undefined,
    new2: undefined,
  };
  function validateCurrent(): void {
    fieldErrors = {
      ...fieldErrors,
      current: current.length < 1 ? 'Please enter your current password' : undefined,
    };
  }
  $: reactive(validateCurrent, [current]);
  function validateNew1(): void {
    fieldErrors = {
      ...fieldErrors,
      new1:
        new1.length < MIN_PASSWORD_LENGTH
          ? `Password must contain at least ${MIN_PASSWORD_LENGTH} characters`
          : undefined,
    };
  }
  $: reactive(validateNew1, [new1]);
  function validateNew2(): void {
    fieldErrors = {
      ...fieldErrors,
      new2: new2.length === 0 ? undefined : new2 === new1 ? undefined : "Paswords don't match",
    };
  }
  $: reactive(validateNew2, [new1, new2]);
  function validateAll(): void {
    validateCurrent();
    validateNew1();
    validateNew2();
  }
  function resetErrors(): void {
    for (const field of fields) {
      fieldErrors[field] = undefined;
    }
  }

  // Error handling
  let submitEnabled = true;
  let submitError: {type: 'api-error'; message: string} | undefined;

  async function submitForm(): Promise<void> {
    submitEnabled = false;

    // Validation
    validateAll();
    const allFieldsValid = Object.values(fieldErrors).every((error) => error === undefined);
    if (!allFieldsValid) {
      console.warn('Some fields are not valid, not submitting form');
      setTimeout(() => (submitEnabled = true), 200);
      return;
    }

    // Send password change request to API
    console.log('Changing password via API');
    let passwordChangeResult;
    try {
      passwordChangeResult = await apiChangePassword(current, new1);
    } catch (error) {
      submitError = {
        type: 'api-error',
        message: ensureError(error).message,
      };
      submitEnabled = true;
      return;
    }

    if (passwordChangeResult.success) {
      // Password change successful! Add flash.
      addFlash({
        message: 'Password changed successfully',
        severity: 'success',
        icon: 'fa-circle-check',
      });

      // Redirect to home
      goto('/');
    } else {
      // Passowrd change failed
      addFlash({
        message: `Password change failed: ${passwordChangeResult.errorDescription}`,
        severity: 'error',
        icon: 'fa-circle-exclamation',
      });
      flashes.update(true);
    }

    submitEnabled = true;
  }

  onMount(() => {
    requireLogin($loginState, `/auth/password/change/`);

    // Reset field errors, so user is not greeted with errors on page load
    resetErrors();
  });
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">{$i18n.t('navigation.home')}</a></li>
    <li><a href="/profile/">{$i18n.t('navigation.profile')}</a></li>
    <li class="is-active"><a href="./" aria-current="page">Change Password</a></li>
  </ul>
</nav>

{#if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title={$i18n.t('common.error--api-error')}
    message={submitError.message}
    showClose={true}
    on:closed={() => (submitError = undefined)}
  />
{/if}

<Flashes bind:this={flashes} />

<h2 class="title is-size-2">Change Password</h2>

<form
  method="post"
  on:submit={(event) => {
    event.preventDefault();
    void submitForm();
  }}
>
  <PasswordFormInputField
    id="current"
    label="Current Password"
    icon="fa-key"
    required={true}
    error={fieldErrors.current !== undefined}
    bind:value={current}
  />
  {#if fieldErrors.current !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.current})}</div>
  {/if}

  <PasswordFormInputField
    id="new1"
    label="New Password"
    icon="fa-asterisk"
    required={true}
    error={fieldErrors.new1 !== undefined}
    bind:value={new1}
  />
  {#if fieldErrors.new1 !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.new1})}</div>
  {/if}

  <PasswordFormInputField
    id="new2"
    label="New Password (repeat)"
    icon="fa-asterisk"
    required={true}
    error={fieldErrors.new2 !== undefined}
    bind:value={new2}
  />
  {#if fieldErrors.new2 !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.new2})}</div>
  {/if}

  <div class="field">
    <div class="control">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">
        {$i18n.t('common.action--submit')}
      </button>
    </div>
  </div>
</form>

<style>
  .field-error {
    color: #ff3860;
    font-size: 0.8em;
    margin-top: -12px;
    margin-bottom: 12px;
  }
</style>
