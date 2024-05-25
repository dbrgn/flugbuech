<script lang="ts">
  import {onMount} from 'svelte';

  import {ensureError} from '$lib/assert';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
  import SubstitutableText from '$lib/components/SubstitutableText.svelte';
  import {MIN_PASSWORD_LENGTH} from '$lib/constants';
  import {i18n} from '$lib/i18n';
  import {addFlash, refreshLoginState} from '$lib/stores';
  import {reactive} from '$lib/svelte';

  import {goto} from '$app/navigation';

  import {apiRegister} from './api';

  // Form values
  let username: string = '';
  let email: string = '';
  let password1: string = '';
  let password2: string = '';
  let newsletter: boolean = false;

  // Element bindings
  let flashes: Flashes;

  // Validation
  const fields = ['username', 'email', 'password1', 'password2'] as const;
  let fieldErrors: Record<(typeof fields)[number], string | undefined> = {
    username: undefined,
    email: undefined,
    password1: undefined,
    password2: undefined,
  };
  function validateUsername(): void {
    fieldErrors = {
      ...fieldErrors,
      username:
        username.length < 3 ? $i18n.t('auth.error--username-too-short', {count: 3}) : undefined,
    };
  }
  $: reactive(validateUsername, [username]);
  function validateEmail(): void {
    fieldErrors = {
      ...fieldErrors,
      email: email.length < 1 ? $i18n.t('auth.error--missing-email') : undefined,
    };
  }
  $: reactive(validateEmail, [email]);
  function validatePassword1(): void {
    fieldErrors = {
      ...fieldErrors,
      password1:
        password1.length < MIN_PASSWORD_LENGTH
          ? $i18n.t('auth.error--password-too-short', {count: MIN_PASSWORD_LENGTH})
          : undefined,
    };
  }
  $: reactive(validatePassword1, [password1]);
  function validatePassword2(): void {
    fieldErrors = {
      ...fieldErrors,
      password2:
        password2.length === 0
          ? undefined
          : password2 === password1
            ? undefined
            : $i18n.t('auth.error--password-dont-match'),
    };
  }
  $: reactive(validatePassword2, [password1, password2]);
  function validateAll(): void {
    validateUsername();
    validateEmail();
    validatePassword1();
    validatePassword2();
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

    // Send registration request to API
    console.log('Registering via API');
    let registrationResult;
    try {
      registrationResult = await apiRegister(username, email, password1, newsletter);
    } catch (error) {
      submitError = {
        type: 'api-error',
        message: ensureError(error).message,
      };
      submitEnabled = true;
      return;
    }

    if (registrationResult.success) {
      // Registration successful! Add flash.
      addFlash({
        message: $i18n.t('auth.prose--registration-successful'),
        severity: 'success',
        icon: 'fa-circle-check',
      });

      // Refresh login state
      refreshLoginState();

      // Redirect to home
      goto('/');
    } else {
      // Registration failed
      addFlash({
        message: $i18n.t('auth.error--registration-failed', {
          message: registrationResult.errorDescription,
        }),
        severity: 'error',
        icon: 'fa-circle-exclamation',
      });
      flashes.update(true);
    }

    submitEnabled = true;
  }

  onMount(() => {
    // Reset field errors, so user is not greeted with errors on page load
    resetErrors();
  });
</script>

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

<h2 class="title is-size-2">{$i18n.t('auth.title--registration')}</h2>

<p class="content">
  <SubstitutableText text={$i18n.t('auth.prose--already-have-an-account')}>
    <a slot="1" href="/auth/login/" let:text>{text}</a>
  </SubstitutableText>
</p>

<form
  method="post"
  on:submit={(event) => {
    event.preventDefault();
    void submitForm();
  }}
>
  <div class="field">
    <label class="label" for="username">{$i18n.t('auth.title--username')}</label>
    <div class="control has-icons-left">
      <!-- svelte-ignore a11y-autofocus -->
      <input
        id="username"
        type="text"
        class="input"
        class:error={fieldErrors.username !== undefined}
        required
        autofocus
        bind:value={username}
      />
      <span class="icon is-small is-left">
        <i class="fas fa-user"></i>
      </span>
    </div>
  </div>
  {#if fieldErrors.username !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.username})}</div>
  {/if}

  <div class="field">
    <label class="label" for="email">{$i18n.t('auth.title--email')}</label>
    <div class="control has-icons-left">
      <input
        id="email"
        type="text"
        class="input"
        class:error={fieldErrors.email !== undefined}
        required
        bind:value={email}
      />
      <span class="icon is-small is-left">
        <i class="fas fa-envelope"></i>
      </span>
    </div>
  </div>
  {#if fieldErrors.email !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.email})}</div>
  {/if}

  <div class="field">
    <label class="label" for="password1">{$i18n.t('auth.title--password')}</label>
    <div class="control has-icons-left">
      <input
        id="password1"
        type="password"
        class="input"
        class:error={fieldErrors.password1 !== undefined}
        required
        placeholder={$i18n.t('auth.prose--choose-password', {count: MIN_PASSWORD_LENGTH})}
        bind:value={password1}
      />
      <span class="icon is-small is-left">
        <i class="fas fa-lock"></i>
      </span>
    </div>
  </div>
  {#if fieldErrors.password1 !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.password1})}</div>
  {/if}

  <div class="field">
    <label class="label" for="password2">{$i18n.t('auth.title--password-repeat')}</label>
    <div class="control has-icons-left">
      <input
        id="password2"
        type="password"
        class="input"
        class:error={fieldErrors.password2 !== undefined}
        required
        placeholder={$i18n.t('auth.prose--password-repeat')}
        bind:value={password2}
      />
      <span class="icon is-small is-left">
        <i class="fas fa-lock"></i>
      </span>
    </div>
  </div>
  {#if fieldErrors.password2 !== undefined}
    <div class="field-error">{$i18n.t('common.error', {message: fieldErrors.password2})}</div>
  {/if}

  <div class="field newsletter">
    <div class="control has-icons-left">
      <label class="checkbox" for="newsletter">
        <input id="newsletter" type="checkbox" bind:checked={newsletter} />
        {$i18n.t('auth.prose--newsletter')}
      </label>
    </div>
  </div>

  <p class="content privacy-policy-hint">
    <SubstitutableText text={$i18n.t('auth.prose--privacy-policy-acknowledge')}>
      <a slot="1" href="/privacy-policy/" target="_blank" let:text>{text}</a>
    </SubstitutableText>
  </p>
  <div class="field">
    <div class="control">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">
        {$i18n.t('auth.action--register')}
      </button>
    </div>
  </div>
</form>

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

  .field.newsletter {
    margin-top: 2em;
  }

  .privacy-policy-hint {
    margin-top: 2em;
    font-style: italic;
  }
</style>
