/**
 * Some code to make the hamburger menu work.
 */

window.ready(() => { // `ready`-function is defined in common.js
    // Get "navbar-burger" element
    const el = document.getElementById('burger-menu-button');

    // Add a click event listener
    if (el !== null) {
        el.addEventListener('click', () => {
            // Get the target from the "data-target" attribute
            const target = document.getElementById(el.dataset.target);

            // Toggle the "is-active" class on both the "navbar-burger" and the "navbar-menu"
            el.classList.toggle('is-active');
            target.classList.toggle('is-active');

            // Toggle the aria attributes
            el.setAttribute('aria-expanded', el.classList.contains('is-active') ? 'true' : 'false');
        });
    }
});
