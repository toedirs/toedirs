![Tests](https://github.com/toedirs/toedirs/actions/workflows/test.yaml/badge.svg)

# Toedi fitness tracking platform

Toedi is a self-hosted fitness tracking and workout planning app.

See the [documentation](https://gettoedi.ch/) for more details.

![metrics](docs/metrics.png)
![calendar](docs/calendar.png)

## Running the project

```bash
docker compose -f docker-compose.yml up
```

## Developing locally

### Installing dependencies

```bash
rustup toolchain install nightly --allow-downgrade
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos
```

### Running for local development

```bash
docker compose -f docker-compose.dev.yml up &
export DATABASE_URL=postgres://toedi:toedi@localhost::5432/toedi
cargo leptos watch
```

## Creating prepared sqlx queries

This is needed to build when there is not database available, as sqlx does compile time checking against a database

```bash
cargo sqlx prepare -- --all-targets --all-features --release
```

## Compiling for Release
```bash
cargo leptos build --release
```

Will generate your server binary in target/server/release and your site package in target/site

