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

<!-- ## Testing the Project -->
<!-- ```bash -->
<!-- cargo leptos end-to-end -->
<!-- ``` -->

<!-- ```bash -->
<!-- cargo leptos end-to-end --release -->
<!-- ``` -->

<!-- Cargo-leptos uses Playwright as the end-to-end test tool. -->  
<!-- Tests are located in end2end/tests directory. -->

<!-- ## Executing a Server on a Remote Machine Without the Toolchain -->
<!-- After running a `cargo leptos build --release` the minimum files needed are: -->

<!-- 1. The server binary located in `target/server/release` -->
<!-- 2. The `site` directory and all files within located in `target/site` -->

<!-- Copy these files to your remote server. The directory structure should be: -->
<!-- ```text -->
<!-- toedirs -->
<!-- site/ -->
<!-- ``` -->
<!-- Set the following environment variables (updating for your project as needed): -->
<!-- ```text -->
<!-- LEPTOS_OUTPUT_NAME="toedirs" -->
<!-- LEPTOS_SITE_ROOT="site" -->
<!-- LEPTOS_SITE_PKG_DIR="pkg" -->
<!-- LEPTOS_SITE_ADDR="127.0.0.1:3000" -->
<!-- LEPTOS_RELOAD_PORT="3001" -->
<!-- ``` -->
<!-- Finally, run the server binary. -->
