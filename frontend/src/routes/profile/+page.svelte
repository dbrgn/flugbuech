<script lang="ts">
  import Flashes from '$lib/components/Flashes.svelte';

  import type {Data} from './+page';

  export let data: Data;

  let flashes: Flashes;
</script>

<nav class="breadcrumb" aria-label="breadcrumbs">
  <ul>
    <li><a href="/">Home</a></li>
    <li class="is-active"><a href="./" aria-current="page">Profile</a></li>
  </ul>
</nav>

<Flashes bind:this={flashes} />

<h2 class="title is-2">Profile</h2>

<p class="content">
  Welcome, {data.profile.username}! Here you can view and update your user profile.
</p>

<p class="content">
  If you want to change your login password,
  <a href="/auth/password/change/">click here</a>.
</p>

<div class="container spaced-headers">
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
