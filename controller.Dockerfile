# STAGE1: Build the binary
FROM rust:alpine as builder
RUN apk add --no-cache musl-dev

# Create a new empty shell project
WORKDIR /app/controller

# Copy over the Cargo.toml files and required local direct and indirect dependencies
COPY controller/Cargo.* .
COPY datum /app/datum
COPY device /app/device
COPY actuator /app/actuator
COPY actuator_temperature /app/actuator_temperature
COPY sensor /app/sensor
COPY sensor_temperature /app/sensor_temperature

# Build and cache the dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

# Copy the actual code files and build the application
COPY controller/src src
# Update the file date
RUN touch src/main.rs
RUN cargo build --release

# STAGE2: create a slim image with the compiled binary
FROM alpine as runner

# Copy the binary from the builder stage
WORKDIR /app
COPY --from=builder /app/controller/target/release/controller app

# TODO this should be configurable
EXPOSE 6565

ENTRYPOINT ["./app", "container"]