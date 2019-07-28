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

## Testing

First, set up a test database:

    createdb flugbuech_test

Run tests:

    cargo test

## License

Licensed under the AGPL version 3 or later. See `LICENSE.md` file.

    Copyright (C) 2019 Danilo Bargen

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
