# realm-icons image: one binary (Rust server + embedded web UI + embedded
# glyph sets) on distroless. A plain `docker build .` works; nothing
# BuildKit-only is used. Build context is the repo root; .dockerignore
# narrows it to server/ and web/.

# --- web: static UI -----------------------------------------------------------
FROM node:24-alpine AS web
RUN corepack enable && corepack prepare pnpm@10.13.1 --activate
WORKDIR /web
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm build

# --- server: the binary -------------------------------------------------------
FROM rust:1.96-bookworm AS server
WORKDIR /app
COPY server/ ./server/
# rust-embed reads ../web/dist relative to the crate at compile time.
COPY --from=web /web/dist ./web/dist
RUN cd server && cargo build --release --locked && cp target/release/realm-icons /realm-icons

# --- runtime: the binary and glibc, nothing else --------------------------------
FROM gcr.io/distroless/cc-debian12
COPY --from=server /realm-icons /app/realm-icons
ENV PORT=3000
EXPOSE 3000
# No shell in the image: the binary probes itself.
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s CMD ["/app/realm-icons", "healthcheck"]
ENTRYPOINT ["/app/realm-icons"]
