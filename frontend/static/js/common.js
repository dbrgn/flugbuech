/**
 * Global helper functions
 */

window.ready = (fn) => {
    if (document.readyState != 'loading'){
        fn();
    } else {
        document.addEventListener('DOMContentLoaded', fn);
    }
};
