# Flugbuech

The Flugbuech (Swiss German for "flight book") application serves as a flight
book / log for free flight pilots (paragliding, hang gliding).

Please see the `README.md` file for more information.

## Build & Commands

### Backend

- Run dev server: `cargo run`
- Run tests: `cargo test`

### Frontend

Located in the `frontend/` directory.

- Run dev server: `npm run dev`
- Run typechecker: `npm run check`
- Run linter: `npm run lint`
- Run unit tests: `npm run test:unit`
- Run integration tests: `npm run test:integration`

## Architecture

### Core Technologies

- **Language**: Rust (backend), TypeScript (frontend)
- **UI Framework**: Svelte
- **Architecture**: Backend with REST API, frontend with SvelteKit
- **Maps**: MapLibre SDK
- **Build System**: Cargo (backend) and Vite (frontend)

## Code Style

- Follow Rust and TypeScript coding conventions
- Backend: Apply Rust code style using `cargo fmt`
- Frontend: Apply Prettier code style using `npm run format`
- Avoid deeply nested logic by using early returns for error cases
- Write clean, high-quality code with concise comments and clear variable names

## Security

- Never commit secrets or API keys to repository

## Decisions

Whenever there is a situation where you need to choose between two approaches,
don't just pick one. Instead, ask.

This includes:

- Choosing between two possible architectural approaches
- Choosing between two libraries to use
...and similar situations.
