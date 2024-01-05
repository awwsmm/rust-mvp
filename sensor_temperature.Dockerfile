# STAGE1: Build the binary
FROM rust:alpine as builder
RUN apk add --no-cache musl-dev

# Create a new empty shell project
WORKDIR /app/sensor_temperature

# Copy over the Cargo.toml files and required local direct and indirect dependencies
COPY sensor_temperature/Cargo.* .
COPY datum /app/datum
COPY device /app/device
COPY sensor /app/sensor

# Build and cache the dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

# Copy the actual code files and build the application
COPY sensor_temperature/src src
# Update the file date
RUN touch src/main.rs
RUN cargo build --release

# STAGE2: create a slim image with the compiled binary
FROM alpine as runner

# Copy the binary from the builder stage
WORKDIR /app
COPY --from=builder /app/sensor_temperature/target/release/sensor_temperature app

# TODO this should be configurable
EXPOSE 8787

ENTRYPOINT ["./app", "container"]