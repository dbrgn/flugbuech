<script lang="ts">
  import {onMount} from 'svelte';

  import CountryFlag from '$lib/components/CountryFlag.svelte';
  import NavbarItem from '$lib/components/NavbarItem.svelte';
  import {changeLanguage, i18n, initializeI18n} from '$lib/i18n';
  import {loginState, refreshLoginState} from '$lib/stores';

  let menuOpened = false;

  function toggleMenu(): void {
    menuOpened = !menuOpened;
  }

  function closeMenu(): void {
    menuOpened = false;
  }

  onMount(() => {
    initializeI18n();
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
      <NavbarItem text={$i18n.t('navigation.home', 'Home')} href="/" {closeMenu} />
      {#if $loginState?.username}
        <NavbarItem
          text={$i18n.t('navigation.gliders', 'My Gliders')}
          href="/gliders/"
          {closeMenu}
        />
        <NavbarItem
          text={$i18n.t('navigation.locations', 'My Locations')}
          href="/locations/"
          {closeMenu}
        />
        <NavbarItem
          text={$i18n.t('navigation.flights', 'My Flights')}
          href="/flights/"
          {closeMenu}
        />
        <NavbarItem text={$i18n.t('navigation.stats', 'Stats')} href="/stats/" {closeMenu} />
        <NavbarItem
          text={$i18n.t('navigation.submit', 'Submit flight')}
          href="/flights/add/"
          {closeMenu}
        />
      {:else}
        <NavbarItem
          text={$i18n.t('navigation.screenshots', 'Screenshots')}
          href="/screenshots/"
          {closeMenu}
        />
      {/if}
    </div>
    <div class="navbar-end">
      <div class="navbar-item language-switcher">
        <a href="." on:click={() => changeLanguage('de')}><CountryFlag countryCode="de" /></a>
        <a href="." on:click={() => changeLanguage('en')}><CountryFlag countryCode="gb" /></a>
      </div>
      <div class="navbar-item">
        <div class="buttons">
          {#if $loginState?.username}
            <NavbarItem
              text={$i18n.t('navigation.profile', 'Profile')}
              href="/profile/"
              type="button"
              {closeMenu}
            />
            <NavbarItem
              text={$i18n.t('navigation.logout', 'Logout')}
              href="/auth/logout/"
              type="button"
              reload={true}
              {closeMenu}
            />
          {:else}
            <NavbarItem
              text={$i18n.t('navigation.login', 'Login')}
              href="/auth/login/"
              type="button"
              {closeMenu}
            />
            <NavbarItem
              text={$i18n.t('navigation.register', 'Register')}
              href="/auth/registration/"
              type="button"
              {closeMenu}
            />
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
      <p class="subtitle">
        {$i18n.t('layout.welcome', 'Welcome, {name}!', {name: $loginState?.username || 'Guest'})}
      </p>
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
    &copy; 2019&ndash;2024 Danilo Bargen | <a href="https://github.com/dbrgn/flugbuech">
      {$i18n.t('layout.sourcecode', 'Source Code')}
    </a>
    |
    <a href="https://github.com/dbrgn/flugbuech/issues">
      {$i18n.t('layout.issue-tracker', 'Issue Tracker')}
    </a>
    |
    <a href="/privacy-policy/">
      {$i18n.t('layout.privacy-policy', 'Privacy Policy')}
    </a>
    |
    <a href="mailto:flugbuech@bargen.dev">
      {$i18n.t('layout.contact', 'Contact')}
    </a>
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

  .language-switcher {
    display: flex;
    flex-direction: row;
    gap: 8px;
    padding-right: 0;
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
