FROM node:20-slim AS frontend-build
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /app/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm run build

FROM rust:latest AS backend-build
ARG DB_BACKEND=sqlite
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev \
    $([ "$DB_BACKEND" = "postgres" ] && echo "libpq-dev") \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --no-default-features --features $DB_BACKEND

FROM debian:trixie-slim
ARG DB_BACKEND=sqlite
RUN apt-get update && apt-get install -y ca-certificates \
    $([ "$DB_BACKEND" = "postgres" ] && echo "libpq5") \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-build /app/target/release/aff-server ./
COPY --from=frontend-build /app/frontend/dist ./frontend/dist
EXPOSE 8080
CMD ["./aff-server"]
