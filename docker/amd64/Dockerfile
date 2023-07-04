# This file was generated using a Jinja2 template.
# Please make your changes in `Dockerfile.j2` and then `make` the individual Dockerfiles.


# Using multistage build:
# 	https://docs.docker.com/develop/develop-images/multistage-build/
# 	https://whitfin.io/speeding-up-rust-docker-builds/

########################## BUILD IMAGE  ##########################
FROM rust:slim-bullseye as build

# Build time options to avoid dpkg warnings and help with reproducible builds.
ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    TZ=UTC \
    TERM=xterm-256color \
    CARGO_HOME="/root/.cargo" \
    USER="root"

# Create CARGO_HOME folder and don't download rust docs
RUN --mount=type=cache,target=/root/.cargo/git --mount=type=cache,target=/root/.cargo/registry mkdir -pv "${CARGO_HOME}" \
    && rustup set profile minimal

# Install build dependencies
RUN apt update \
    && apt install -y \
        --no-install-recommends \
        libssl-dev

# Creates a dummy project used to grab dependencies
RUN USER=root cargo new --bin /app
WORKDIR /app

# Copies over *only* your manifests and build files
COPY ./Cargo.* ./


# Builds your dependencies and removes the
# dummy project, except the target folder
# This folder contains the compiled dependencies
RUN --mount=type=cache,target=/root/.cargo/git --mount=type=cache,target=/root/.cargo/registry cargo build --release \
    && find ./ -not -path "./target*" -delete

# Copies the complete project
# To avoid copying unneeded files, use .dockerignore
COPY ./ ./

# Make sure that we actually build the project
RUN touch src/main.rs

# Builds again, this time it'll just be
# your actual source files being built
RUN --mount=type=cache,target=/root/.cargo/git --mount=type=cache,target=/root/.cargo/registry cargo build --release

######################## RUNTIME IMAGE  ########################
# Create a new stage with a minimal image
# because we already have a binary built
FROM debian:bullseye-slim

RUN apt update && apt install -y \
    --no-install-recommends \
    libssl1.1 \
    && apt clean \
    && rm -rf /var/lib/apt/lists/*


RUN mkdir -p /app/
WORKDIR /app/

# Copies the files from the context (Rocket.toml file and web-vault)
# and the binary from the "build" stage to the current stage
COPY --from=build /app/target/release/resback ./
COPY --from=build /app/.env ./
COPY --from=build /app/private_key.pem ./
COPY --from=build /app/public_key.pem ./

EXPOSE 3000

CMD ["./resback"]
