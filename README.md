# Flugbuech

Flight log for paragliding pilots.

## Setup

Use Rust nightly:

    rustup update nightly
    rustup override add nightly

Install `diesel_cli`:

    cargo install -f diesel_cli --no-default-features --features postgres

Set up PostgreSQL database:

    diesel setup

Start server:

    cargo run
