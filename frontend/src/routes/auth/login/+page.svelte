<script lang="ts">
  import {goto} from '$app/navigation';

  import Flashes from '$lib/components/Flashes.svelte';
  import {addFlash, refreshLoginState} from '$lib/stores';
  import {ensureError} from '$lib/assert';
  import MessageModal from '$lib/components/MessageModal.svelte';

  import {apiLogin} from './api';

  // Form values
  let username: string;
  let password: string;

  // Element bindings
  let flashes: Flashes;

  // Error handling
  let submitEnabled = true;
  let submitError: {type: 'api-error'; message: string} | undefined;

  async function submitForm(): Promise<void> {
    submitEnabled = false;

    // Send login request to API
    let success;
    try {
      success = (await apiLogin(username, password)).success;
    } catch (error) {
      submitError = {
        type: 'api-error',
        message: ensureError(error).message,
      };
      submitEnabled = true;
      return;
    }

    if (success) {
      // Login successful! Add flash.
      addFlash({
        message:
          'Login successful, welcome! Your session will remain active for 1 year, until you log out.',
        severity: 'success',
        icon: 'fa-circle-check',
      });

      // Refresh login state
      refreshLoginState();

      // Redirect to home
      // TODO: Parse redirect target URL
      goto('/');
    } else {
      // Login failed
      addFlash({
        message: 'Login failed. Check your username / password.',
        severity: 'error',
        icon: 'fa-circle-exclamation',
      });
      flashes.update(true);
      password = '';
    }

    submitEnabled = true;
  }
</script>

{#if submitError?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message={submitError.message}
    showClose={true}
    on:closed={() => (submitError = undefined)}
  />
{/if}

<Flashes bind:this={flashes} />

<h2 class="title is-size-2">Login</h2>

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
        class="input"
        type="text"
        name="username"
        id="username"
        bind:value={username}
        required
        autofocus
      />
      <span class="icon is-small is-left">
        <i class="fas fa-user"></i>
      </span>
    </div>
  </div>
  <div class="field">
    <label class="label" for="password">Password</label>
    <div class="control has-icons-left">
      <input
        class="input"
        type="password"
        name="password"
        id="password"
        bind:value={password}
        required
      />
      <span class="icon is-small is-left">
        <i class="fas fa-lock"></i>
      </span>
    </div>
  </div>
  <div class="field">
    <div class="control">
      <button class="button is-primary" disabled={!submitEnabled} type="submit">Login</button>
    </div>
  </div>

  <p>Don't have an account yet? <a href="/auth/registration">Register now!</a></p>
</form>
