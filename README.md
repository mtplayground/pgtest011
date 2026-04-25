# pgtest011

TodoMVC-style full-stack app built with Leptos, Axum, PostgreSQL, and SQLx.

The app serves a server-rendered Leptos UI with client hydration, persists all todo
state in PostgreSQL, and exposes both browser routes and JSON API endpoints.

## Requirements

- Rust 1.95 or newer
- `cargo-leptos`
- `wasm32-unknown-unknown` Rust target
- Node.js and npm
- `sass` available on `PATH`
- PostgreSQL 16 or any compatible PostgreSQL instance
- Docker and Docker Compose if you want the container workflow

One-time local setup:

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
npm install -g sass
```

## Quick Start

1. Copy the example environment file:

```bash
cp .env.example .env
```

2. Update `DATABASE_URL` to point at a PostgreSQL database.

3. Start the app in development mode:

```bash
cargo leptos watch
```

4. Open `http://127.0.0.1:8080`.

`cargo-leptos watch` builds both the SSR server and the hydrated WASM client and
reloads when source files change.

## Environment Variables

The app reads configuration from the environment via `dotenvy`. These variables are
recognized:

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `DATABASE_URL` | Yes | none | PostgreSQL connection string used by SQLx and startup migrations. |
| `APP_ENV` | No | `development` | Free-form environment label used in logs. |
| `HOST` | No | `0.0.0.0` | Bind host for the Axum server. |
| `PORT` | No | `8080` | Bind port for the Axum server. |
| `RUST_LOG` | No | `pgtest011=info,tower_http=info,axum::rejection=trace` | Tracing filter. |

Example:

```env
APP_ENV=development
HOST=0.0.0.0
PORT=8080
RUST_LOG=pgtest011=info,tower_http=info,axum::rejection=trace
DATABASE_URL=postgres://postgres:postgres@localhost:5432/pgtest011
```

## Development Workflow

Run the live-reload dev server:

```bash
cargo leptos watch
```

Run a release build:

```bash
cargo build --release
```

Run the Rust test suites:

```bash
cargo test
```

List the Playwright TodoMVC end-to-end tests:

```bash
npx playwright test --config e2e/playwright.config.ts --list
```

## Docker Compose

The repository includes a two-service Docker Compose stack:

- `postgres`: PostgreSQL 16 with a named `postgres_data` volume
- `app`: the Leptos + Axum application built from the included multi-stage Dockerfile

Start the stack:

```bash
docker compose up --build
```

The app will be available at `http://127.0.0.1:8080` and the container health check
targets `GET /healthz`.

Stop the stack:

```bash
docker compose down
```

Remove the database volume as well:

```bash
docker compose down -v
```

## Deployment Notes

- The container image is built with `cargo leptos build --release`.
- The runtime container listens on `0.0.0.0:8080`.
- Startup runs embedded SQLx migrations before serving requests.
- Persistent state must use PostgreSQL. Do not replace it with SQLite, JSON files,
  or in-memory storage.

For production-style local testing with the repo’s runtime defaults:

```bash
export DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/pgtest011
cargo leptos serve --release
```

## Troubleshooting

`missing required environment variable DATABASE_URL`

Set `DATABASE_URL` in your shell or `.env` before starting the app.

`could not resolve bind address`

Check that `HOST` and `PORT` form a valid bind address.

`cargo leptos: command not found`

Install it with `cargo install cargo-leptos --locked`.

Hydration build fails because `sass` is missing

Install Sass globally with `npm install -g sass`.

Database connection or migration failures at startup

Confirm the PostgreSQL server is reachable, the credentials in `DATABASE_URL` are
correct, and the target database exists.

Playwright cannot resolve the test runner module

Use the Playwright installation available in the environment, or install the
Playwright test runner locally before executing the browser suite.
