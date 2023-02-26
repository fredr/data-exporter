FROM docker.io/rust:1.67.1 as build
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11:nonroot
COPY --from=build --chown=nonroot:nonroot /app/target/release/data-exporter /
ENTRYPOINT ["/data-exporter"]
