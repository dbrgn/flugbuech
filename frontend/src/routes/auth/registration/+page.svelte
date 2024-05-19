<script lang="ts">
  import {onMount} from 'svelte';

  import {ensureError} from '$lib/assert';
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';
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
      username: username.length < 3 ? 'Username must consist of at least 3 characters' : undefined,
    };
  }
  $: reactive(validateUsername, [username]);
  function validateEmail(): void {
    fieldErrors = {
      ...fieldErrors,
      email: email.length < 1 ? 'Please enter an e-mail address' : undefined,
    };
  }
  $: reactive(validateEmail, [email]);
  function validatePassword1(): void {
    fieldErrors = {
      ...fieldErrors,
      password1:
        password1.length < MIN_PASSWORD_LENGTH
          ? `Password must contain at least ${MIN_PASSWORD_LENGTH} characters`
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
            : "Paswords don't match",
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
        message: 'Registration successful, welcome!',
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
        message: `Registration failed: ${registrationResult.errorDescription}`,
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

<h2 class="title is-size-2">Registration</h2>

<p class="content">Already have an account? <a href="/auth/login">Log in now!</a></p>

<form
  method="post"
  on:submit={(event) => {
    event.preventDefault();
    void submitForm();
  }}
>
  <div class="field">
    <label class="label" for="username">Username</label>
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
    <label class="label" for="email">E-mail</label>
    <div class="control has-icons-left">
      <input
        id="email"
        type="text"
        class="input"
        class:error={fieldErrors.username !== undefined}
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
    <label class="label" for="password1">Password</label>
    <div class="control has-icons-left">
      <input
        id="password1"
        type="password"
        class="input"
        class:error={fieldErrors.password1 !== undefined}
        required
        placeholder="Choose a password (at least {MIN_PASSWORD_LENGTH} characters)"
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
    <label class="label" for="password2">Password Confirmation</label>
    <div class="control has-icons-left">
      <input
        id="password2"
        type="password"
        class="input"
        class:error={fieldErrors.password2 !== undefined}
        required
        placeholder="Confirm password"
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
        I want to receive occasional news about Flugbuech through e-mail
      </label>
    </div>
  </div>

  <p class="content privacy-policy-hint">
    By registering, you acknowledge the <a href="/privacy-policy/" target="_blank">privacy policy</a
    >.
  </p>
  <div class="field">
    <div class="control">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">Register</button>
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
