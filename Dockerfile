# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-bullseye as builder
ARG PROFILE=debug
# If you’re using stable, use this instead
# FROM rust:1.70-bullseye as builder

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

# Install cargo-leptos
RUN cargo binstall cargo-leptos -y

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

# Build the app
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
  if [ "$PROFILE" = "debug" ]; then \
    cargo leptos build -vv ; \
  else \
    cargo leptos build --release -vv ; \
  fi; \
  mkdir output;\
  cp target/${PROFILE}/toedirs output/; \
  cp -r target/site output/site;

FROM rustlang/rust:nightly-bullseye as runner
# FROM scratch as runner
ARG PROFILE=debug
USER 10001
# Copy the server binary to the /app directory
COPY --chown=10001:10001 --from=builder /app/output/toedirs /app/
# /target/site contains our JS/WASM/CSS, etc.
COPY --chown=10001:10001 --from=builder /app/output/site /app/site
# Copy Cargo.toml if it’s needed at runtime
COPY --chown=10001:10001 --from=builder /app/Cargo.toml /app/
COPY --chown=10001:10001 --from=builder /app/toedi.toml /app/
WORKDIR /app

# RUN if [ "$PROFILE" = "debug" ]; then \
#     wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz; \
#     tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz; \
#     cp cargo-binstall /usr/local/cargo/bin; \
#     cargo binstall cargo-leptos -y; \
#     rustup target add wasm32-unknown-unknown; \
#   fi
  
# RUN if [ "$PROFILE" = "debug" ]; then \
#     export APP_ENVIRONMENT="develop"; \
#   else \
#     export APP_ENVIRONMENT="production"; \
#   fi

# Set any required env variables and
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8479"
ENV LEPTOS_SITE_ROOT="site"
ENV DATABASE_URL=
EXPOSE 8479
# Run the server
ENTRYPOINT ["/app/toedirs"]
