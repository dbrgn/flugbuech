<script lang="ts">
  import {onMount} from 'svelte';

  import NavbarItem from '$lib/components/NavbarItem.svelte';
  import {loginState, refreshLoginState} from '$lib/stores';

  let menuOpened = false;

  function toggleMenu(): void {
    menuOpened = !menuOpened;
  }

  function closeMenu(): void {
    menuOpened = false;
  }

  onMount(() => {
    refreshLoginState();
  });
</script>

<!-- Navbar -->
<nav class="navbar" aria-label="main navigation">
  <div class="navbar-brand">
    <div class="navbar-item">
      <span class="icon is-small fa-solid fa-parachute-box"></span>
    </div>

    <a
      role="button"
      id="burger-menu-button"
      class="navbar-burger burger"
      aria-label="menu"
      aria-expanded="false"
      data-target="navbar-contents"
      tabindex="0"
      class:is-active={menuOpened}
      on:click={toggleMenu}
      on:keydown={(event) => {
        if (['Enter', ' '].includes(event.key)) {
          event.stopPropagation();
          event.preventDefault();
          toggleMenu();
        }
      }}
    >
      <span aria-hidden="true"></span>
      <span aria-hidden="true"></span>
      <span aria-hidden="true"></span>
    </a>
  </div>
  <div id="navbar-contents" class="navbar-menu" class:is-active={menuOpened}>
    <div class="navbar-start">
      <NavbarItem text="Home" href="/" {closeMenu} />
      {#if $loginState?.username}
        <NavbarItem text="My Gliders" href="/gliders/" {closeMenu} />
        <NavbarItem text="My Locations" href="/locations/" {closeMenu} />
        <NavbarItem text="My Flights" href="/flights/" {closeMenu} />
        <NavbarItem text="Stats" href="/stats/" {closeMenu} />
        <NavbarItem text="Submit flight" href="/flights/add/" {closeMenu} />
      {:else}
        <NavbarItem text="Screenshots" href="/screenshots/" {closeMenu} />
      {/if}
    </div>
    <div class="navbar-end">
      <div class="navbar-item">
        <div class="buttons">
          {#if $loginState?.username}
            <NavbarItem text="Profile" href="/profile/" type="button" {closeMenu} />
            <NavbarItem
              text="Logout"
              href="/auth/logout/"
              type="button"
              reload={true}
              {closeMenu}
            />
          {:else}
            <NavbarItem text="Login" href="/auth/login/" type="button" {closeMenu} />
            <NavbarItem text="Register" href="/auth/registration/" type="button" {closeMenu} />
          {/if}
        </div>
      </div>
    </div>
  </div>
</nav>

<!-- Header -->
<section class="hero is-medium main-header">
  <header class="hero-body">
    <div class="container">
      <h1 class="title">Flugbuech</h1>
      <p class="subtitle">Welcome, {$loginState?.username || 'Guest'}!</p>
    </div>
  </header>
</section>

<!-- Content -->
<div class="section">
  <div class="container">
    <slot />
  </div>
</div>

<!-- Footer -->
<footer class="section">
  <div class="container">
    &copy; 2019&ndash;2024 Danilo Bargen | <a href="https://github.com/dbrgn/flugbuech"
      >Source Code</a
    >
    | <a href="https://github.com/dbrgn/flugbuech/issues">Bug reports and feature requests</a> |
    <a href="/privacy-policy/">Privacy Policy</a>
    | <a href="mailto:flugbuech@bargen.dev">Contact</a>
  </div>
</footer>

<style>
  .main-header {
    background: linear-gradient(rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.5)), url('/img/bg.jpg');
    background-repeat: no-repeat;
    background-position: center center;
    background-size: cover;
  }

  .main-header h1 {
    font-size: 4em;
    color: #eee;
  }

  .main-header .subtitle {
    color: #eee;
  }

  /** Bulma CSS variable customization */

  :root {
    /* Link color: A more soft, slightly desaturated blue */
    --bulma-link-h: 204.1deg;
    --bulma-link-s: 80%;
    --bulma-link-on-scheme-l: 45%;
    --bulma-active-color-l-delta: -25%;

    /* Primary color: Blue instead of teal */
    --bulma-primary-h: 204.1deg;
    --bulma-primary-s: 85.8%;
    --bulma-primary-l: 50.4%;
  }

  :global(.button) {
    /* Button: Slightly less padding */
    --bulma-button-padding-vertical: 0.4em;
    --bulma-button-padding-horizontal: 0.75em;
  }

  :global(.button.is-primary),
  :global(.button.is-danger) {
    /* Button: White text instead of black */
    --bulma-button-color-l: 100%;
  }

  :global(.message.is-primary),
  :global(.message.is-danger) {
    /* Message header: White text instead of black */
    --bulma-message-header-color-l: 100%;
  }

  :global(.modal-background) {
    /* Slightly less dark modal background */
    --bulma-modal-background-background-color: hsla(
      var(--bulma-scheme-h),
      var(--bulma-scheme-s),
      var(--bulma-scheme-invert-l),
      0.78
    );
  }
</style>
