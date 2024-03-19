<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';
  import MessageModal from '$lib/components/MessageModal.svelte';

  import {invalidateAll} from '$app/navigation';

  import type {Data} from './+page';
  import {apiUpdateProfile} from './api';

  export let data: Data;

  let flashes: Flashes;

  // Error handling
  let errorModal: {type: 'api-error'; message: string} | undefined;

  function updateNewsOptIn(optIn: boolean): void {
    apiUpdateProfile({newsOptIn: optIn})
      .then((result) => {
        if (result.success) {
          invalidateAll();
        } else {
          errorModal = {
            type: 'api-error',
            message: `Failed to change news status: ${result.errorReason} (${result.errorDescription})`,
          };
        }
      })
      .catch((error) => {
        console.error('Profile update error', error);
        errorModal = {
          type: 'api-error',
          message: `Failed to change news status: ${error?.body?.message !== undefined ? error.body.message : error}`,
        };
      });
  }
</script>

{#if errorModal?.type === 'api-error'}
  <MessageModal
    type="error"
    title="API Error"
    message={errorModal.message}
    showClose={true}
    on:closed={() => (errorModal = undefined)}
  />
{/if}

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Profile</a></li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">Profile</h2>

<p class="block">
  Welcome, <strong>{data.profile.username}</strong>! Here you can view and update your user profile.
</p>

<p class="block">
  If you want to change your login password,
  <a href="/auth/password/change/">click here</a>.
</p>

<div class="block">
  <h3 class="title is-4">User Info</h3>
  <table class="table is-hoverable">
    <tbody>
      <tr>
        <th>Username</th>
        <td>{data.profile.username}</td>
      </tr>
      <tr>
        <th>E-mail</th>
        <td>{data.profile.email}</td>
      </tr>
      <tr>
        <th>Registered Since</th>
        <td>{data.profile.signedUp}</td>
      </tr>
    </tbody>
  </table>
</div>

<div class="block">
  <h3 class="title is-4">News</h3>

  <article class="message is-info">
    <div class="message-body">
      If you agree, I will send you occasional, tracking-free e-mails with news about Flugbuech.
      These e-mails are purely about aspects of the flight book software itself. There will probably
      be only 1-3 e-mails per year. You can unsubscribe here at any time.
    </div>
  </article>

  <div class="content">
    <p>
      You currently <strong>{data.profile.newsOptIn ? 'allow' : "don't want"}</strong> news e-mails.
    </p>
  </div>

  <div class="block">
    <button
      class="button"
      class:is-warning={data.profile.newsOptIn}
      class:is-primary={!data.profile.newsOptIn}
      on:click={() => updateNewsOptIn(!data.profile.newsOptIn)}
    >
      {data.profile.newsOptIn ? 'Unsubscribe from news' : 'Subscribe to news'}
    </button>
  </div>
</div>
