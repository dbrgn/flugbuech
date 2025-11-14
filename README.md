# Flugbuech

[![Build status: Backend](https://github.com/dbrgn/flugbuech/actions/workflows/backend.yml/badge.svg)](https://github.com/dbrgn/flugbuech/actions/workflows/backend.yml)
[![Build status: Frontend](https://github.com/dbrgn/flugbuech/actions/workflows/frontend.yml/badge.svg)](https://github.com/dbrgn/flugbuech/actions/workflows/frontend.yml)
[![Docker image](https://img.shields.io/badge/docker%20image-dbrgn%2Fflugbuech-blue)](https://hub.docker.com/r/dbrgn/flugbuech/)

Personal flight log for paragliding pilots. Written with
[Rust](https://www.rust-lang.org/) / [Rocket](https://rocket.rs/) in
the backend and Bulma CSS / SvelteKit in the frontend.

![Screenshot](screenshot.png)

This software is mostly aimed at pilots that track their flights using a GPS
tracker which generates IGC files, but it can also be used without an IGC file
for every flight.

The IGC file is stored together with the flight data. Every flight can be
linked to an [XContest](https://www.xcontest.org/) upload.

A free hosted version of the flight book can be found at
[flugbue.ch](https://flugbue.ch/). Registration is open!

If you have any question, you can contact me at
[flugbuech@bargen.dev](mailto:flugbuech@bargen.dev).


## Status

### Features

- Add flights manually to the database
- Add flights through IGC file upload (all relevant flight data like launch
  time/location, landing time/location, duration, distance, etc can be
  extracted from that file)
- Add/edit/delete launch/landing locations
- Map of all locations
- Add/edit/delete gliders/wings
- Stats about the past flights
- Import flights from CSV
- Translation support: English and German

See <https://flugbue.ch/screenshots/> for a good overview.

### What's not yet implemented

- [ ] Password recovery
- [ ] Showing flight track on a map
- [ ] Map with all your flights
- [ ] Make it easier to self-host
- [ ] Import launch / landing locations from a public database
- [ ] Direct upload to XContest


## Setup

Requirements:

- Rust stable (see `rust-toolchain` file)
- PostgreSQL + PostGIS
- For building the frontend components: nodejs / npm
- For production environments: A reverse proxy like Nginx

Install `diesel_cli`:

    cargo install -f diesel_cli --no-default-features --features postgres

Start PostGIS database:

    docker run -d --name flugbuech-pg \
      -e POSTGRES_DB=flugbuech \
      -e POSTGRES_USER=flugbuech \
      -e POSTGRES_PASSWORD=flugbuech-dev-password \
      -p 127.0.0.1:5432:5432 \
      docker.io/postgis/postgis:15-3.4-alpine

Prepare database and run migrations:

    diesel setup

(Note: You can also apply migrations by starting with the `--migrate` flag)

Install frontend dependencies:

    (cd frontend && npm install)

You're set up! To start the development API server:

    cargo run

And to run the development frontend server in a second terminal window:

    cd frontend && npm run dev

Then go to <http://localhost:5173/> to view the local application.


## Creating test users

Head to <http://localhost:5173/auth/registration> and sign up regularly.


## Testing

First, set up a test database:

    docker exec flugbuech-pg createdb -U flugbuech flugbuech_test

Run backend tests:

    cargo test

To run frontend tests:

    cd frontend
    npm run test


## Resetting Password

To reset a password directly in the database:

    UPDATE users
    SET password = crypt('newpassword', gen_salt('bf', 10))
    WHERE username = 'user';


## Deployment

If you want to deploy this software using Docker, please take a look at the
provided `docker-compose.yml` file. It can serve as an example on how to host
Flugbuech. The image is also available [on Docker
Hub](https://hub.docker.com/r/dbrgn/flugbuech/), images are re-built at least
weekly.

**IMPORTANT**: Make sure to change the `ROCKET_SECRET_KEY` variable when
configuring your deployment!


## License

Licensed under the AGPL version 3 or later. See `LICENSE.md` file.

    Copyright (C) 2019–2025 Danilo Bargen

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
