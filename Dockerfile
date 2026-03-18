# Stage 1 — Build frontend
FROM node:22-alpine AS frontend
WORKDIR /app
COPY frontend/package.json ./
RUN npm install --legacy-peer-deps
COPY frontend/ ./
RUN npm run build

# Stage 2 — Build backend
FROM rust:alpine AS backend
RUN apk add --no-cache musl-dev
WORKDIR /app

# Cache dependencies first by building a dummy binary
COPY backend/Cargo.toml ./
COPY backend/Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN SQLX_OFFLINE=true cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy real source, sqlx cache, and migrations
COPY backend/src ./src
COPY backend/.sqlx ./.sqlx
COPY backend/migrations ./migrations

# Build the real binary
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && cargo build --release

# Stage 3 — Minimal runtime
FROM alpine:3.20
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=backend /app/target/release/jirs ./
COPY --from=backend /app/migrations ./migrations
COPY --from=frontend /app/build ./static
EXPOSE 8080
CMD ["./jirs"]
