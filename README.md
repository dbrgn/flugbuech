# Flugbuech

Personal flight log for paragliding pilots. Written with
[Rust](https://www.rust-lang.org/) / [Rocket](https://rocket.rs/) in
the backend and Bulma CSS / Vanilla JavaScript in the frontend.

This software is mostly aimed at pilots that track their flights using a GPS
tracker which generates IGC files, but it can also be used without an IGC file
for every flight.

I plan to provide a free hosted version of the flight book, but it should also
be possible to host it yourself, either on a server or on your own local
computer.

The IGC file is stored together with the flight data. Every flight can be
linked to an XContest upload.

## Status

Right now this software is still under active development. There has been no
release so far.

### What works

- [x] User login / authentication
- [x] Adding flights to the database
- [x] Simple submission of of flights by uploading IGC file (all
  relevant flight data like launch time/location, landing time/location,
  duration, distance, etc can be extracted from that file)
- [x] Manage locations

### What's not yet implemented

- [ ] User registration, password recovery
- [ ] Map with all your flights
- [ ] Show stats about the past flights
- [ ] Host the page on a public website
- [ ] Make it easy to self-host
- [ ] Import launch / landing locations from a public database
- [ ] Adding links to other XC platforms like [XCR](https://xc-paragliding.com/)

## Setup

Requirements:

    - Rust nightly
    - PostgreSQL + PostGIS
    - For production environments: A reverse proxy like Nginx

Use Rust nightly:

    rustup update nightly
    rustup override add nightly

Install `diesel_cli`:

    cargo install -f diesel_cli --no-default-features --features postgres

Set up PostgreSQL database:

    diesel setup

Start server:

    cargo run

## Adding Users

Right now users have to be added to the database manually.

```sql
INSERT INTO users(username, password)
VALUES ('user', crypt('password', gen_salt('bf', 10)));
```

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

## Credits

- Background photo by [eberhard grossgasteiger on Unsplash](https://unsplash.com/photos/15KSFB1n0FU)
