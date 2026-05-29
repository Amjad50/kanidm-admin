# syntax=docker/dockerfile:1.6
#
# Multi-stage build for kanidm-admin-ui.
#
#   Stage 1 (assets):   bun bundles Tailwind CSS + the islands TS into static/.
#   Stage 2 (rust):     cargo builds the binary with the static/ baked into
#                       the image via include_dir at runtime serving.
#   Stage 3 (runtime):  slim debian with just the binary + static assets.

# ─── Stage 1: build CSS + JS bundles ─────────────────────────────────────────
FROM oven/bun:1.3-debian AS assets

WORKDIR /src

# Cache dep resolution by copying package files first.
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Then copy the sources Tailwind/the bundler actually scans.
COPY tsconfig.json ./
COPY styles ./styles
COPY templates ./templates
COPY islands ./islands
COPY scripts ./scripts
# The icon generator writes into src/, so we need src to exist before bun runs.
COPY src ./src
# Source-tracked static assets (favicon, logos). .dockerignore excludes the
# bun-built app.css/app.js so this only brings the SVGs and similar; bun
# will write its outputs alongside them into the same directory.
COPY static ./static

RUN bun run build

# ─── Stage 2: build the Rust binary ──────────────────────────────────────────
FROM rust:1-bookworm AS rust-build

WORKDIR /src

# Pre-fetch deps using Cargo's dummy-source trick so dep compilation is
# cached across source-only changes.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs \
 && cargo build --release \
 && rm -rf src target/release/deps/kanidm_admin_ui*

COPY src ./src
COPY templates ./templates
# Copy the freshly built static/ from stage 1 so any compile-time
# embedding (askama, etc.) sees the real assets.
COPY --from=assets /src/static ./static

RUN cargo build --release --locked
RUN strip target/release/kanidm-admin-ui

# ─── Stage 3: minimal runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --user-group --no-create-home --uid 1000 kanidm-admin-ui

WORKDIR /app

COPY --from=rust-build /src/target/release/kanidm-admin-ui /usr/local/bin/kanidm-admin-ui
COPY --from=assets /src/static /app/static

USER kanidm-admin-ui
EXPOSE 3000

ENV KANIDM_ADMIN_BIND_ADDR=0.0.0.0:3000 \
    KANIDM_ADMIN_STATIC_DIR=/app/static

CMD ["/usr/local/bin/kanidm-admin-ui"]
