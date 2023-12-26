<!-- Navbar -->
<nav class="navbar" role="navigation" aria-label="main navigation">
  <div class="navbar-brand">
    <div class="navbar-item">
      <span class="icon is-small fas fa-parachute-box"></span>
    </div>

    <a
      role="button"
      id="burger-menu-button"
      class="navbar-burger burger"
      aria-label="menu"
      aria-expanded="false"
      data-target="navbar-contents"
    >
      <span aria-hidden="true"></span>
      <span aria-hidden="true"></span>
      <span aria-hidden="true"></span>
    </a>
  </div>
  <div id="navbar-contents" class="navbar-menu">
    <div class="navbar-start">
      <a class="navbar-item" href="/">Home</a>
      TODO if user
      <a class="navbar-item" href="/gliders/">My Gliders</a>
      <a class="navbar-item" href="/locations/">My Locations</a>
      <a class="navbar-item" href="/flights/">My Flights</a>
      <a class="navbar-item" href="/stats/">Stats</a>
      <a class="navbar-item" href="/flights/add/">Submit flight</a>
    </div>
    <div class="navbar-end">
      <div class="navbar-item">
        <div class="buttons">
          TODO if user
          <a class="button" href="/profile/">Profile</a>
          <a class="button" href="/auth/logout/">Logout</a>
          TODO else
          <a class="button is-light" href="/auth/login/">Login</a>
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
      <p class="subtitle">Welcome, TODO!</p>
    </div>
  </header>
</section>

<!-- Content -->
<!-- TODO <div class="section {% block sectionclass %}{% endblock %}"> -->
<div class="section">
  <div class="container">
    <slot />
  </div>
</div>

<!-- Footer -->
<footer class="section">
  <div class="container">
    &copy; 2019&ndash;2023 Danilo Bargen | <a href="https://github.com/dbrgn/flugbuech"
      >Source Code</a
    >
    | <a href="https://github.com/dbrgn/flugbuech/issues">Bug reports and feature requests</a> |
    <a href="/privacy-policy">Privacy Policy</a>
    | <a href="mailto:flugbuech@bargen.dev">Contact</a>
  </div>
</footer>
