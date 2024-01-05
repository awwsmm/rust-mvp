# STAGE1: Build the binary
FROM rust:alpine as builder
RUN apk add --no-cache musl-dev

# Create a new empty shell project
WORKDIR /app/environment

# Copy over the Cargo.toml files and required local direct and indirect dependencies
COPY environment/Cargo.* .
COPY actuator_temperature /app/actuator_temperature
COPY datum /app/datum
COPY device /app/device
COPY actuator /app/actuator

# Build and cache the dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

# Copy the actual code files and build the application
COPY environment/src src
# Update the file date
RUN touch src/main.rs
RUN cargo build --release

# STAGE2: create a slim image with the compiled binary
FROM alpine as runner

# Copy the binary from the builder stage
WORKDIR /app
COPY --from=builder /app/environment/target/release/environment app

# TODO this should be configurable
EXPOSE 5454

ENTRYPOINT ["./app", "container"]