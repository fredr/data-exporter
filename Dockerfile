FROM rust:1.56.0 as build
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc:nonroot
COPY --from=build --chown=nonroot:nonroot /app/target/release/data-exporter /
ENTRYPOINT ["/data-exporter"]
