# Flugbuech

Flight log for paragliding pilots.

Goals:

- [ ] Allow users to add flights to the database
- [ ] Show stats about the past flights
- [ ] Simple submission of of flights by uploading IGC file

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
