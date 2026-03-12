FROM rust:1.91-slim AS chef
RUN cargo install cargo-chef cargo-leptos
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo leptos build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/samete /app/samete
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/
WORKDIR /app
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["/app/samete"]
