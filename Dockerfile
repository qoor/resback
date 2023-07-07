# Generic Rust builder for multiple architectures.
#
# This provides a build environment for building Rust codes.
# All flows in this image should be generic.
# You cannot define architecture dependent behaviors.
FROM --platform=${BUILDPLATFORM} rust:slim-bullseye as builder

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    TERM=xterm-256color \
    CARGO_HOME="/root/.cargo" \
    USER="root"

# Create CARGO_HOME folder and don't download rust docs
RUN mkdir -pv "${CARGO_HOME}" \
    && rustup set profile minimal

# Install required dependencies
RUN apt update \
    && apt install -y \
    # These should always on top.
    # After adding the amd64 architecture, the arm64 version of `apt` gives
    # package dependency errors when installing packages for cross compilation.
    gcc-x86-64-linux-gnu libc6-dev-amd64-cross \
    gcc-aarch64-linux-gnu libc6-dev-arm64-cross \
    # Add the architectures we are targeting
    && dpkg --add-architecture amd64 \
    && dpkg --add-architecture arm64 \
    # Install the openssl library with headers
    && apt update && apt install -y \
    libssl-dev:amd64 libssl-dev:arm64 \
    #
    # Make sure cargo has the right target config
    && echo '[target.x86_64-unknown-linux-gnu]' >> "${CARGO_HOME}/config" \
    && echo 'linker = "x86_64-linux-gnu-gcc"' >> "${CARGO_HOME}/config" \
    && echo 'rustflags = ["-L/usr/lib/x86_64-linux-gnu"]' >> "${CARGO_HOME}/config" \
    #
    && echo '[target.aarch64-unknown-linux-gnu]' >> "${CARGO_HOME}/config" \
    && echo 'linker = "aarch64-linux-gnu-gcc"' >> "${CARGO_HOME}/config" \
    && echo 'rustflags = ["-L/usr/lib/aarch64-linux-gnu"]' >> "${CARGO_HOME}/config"

RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu

# Set platform specific environment values
ENV CC_x86_64_unknown_linux_gnu="/usr/bin/x86_64-linux-gnu-gcc" \
    OPENSSL_INCLUDE_DIR="/usr/include/x86_64-linux-gnu" \
    OPENSSL_LIB_DIR="/usr/lib/x86_64-linux-gnu"
ENV CC_aarch64_unknown_linux_gnu="/usr/bin/aarch64-linux-gnu-gcc" \
    OPENSSL_INCLUDE_DIR="/usr/include/aarch64-linux-gnu" \
    OPENSSL_LIB_DIR="/usr/lib/aarch64-linux-gnu"
ENV SQLX_OFFLINE true

WORKDIR /usr/src/resback

# Creates a dummy project used to grab dependencies
RUN USER=root cargo init --bin

# Copies over *only* your manifests and build files
COPY ./Cargo.* ./


#
# Rust amd64 builder
#
FROM --platform=${BUILDPLATFORM} builder as builder-amd64

# Builds your dependencies and removes the dummy project,
# except the target folder
# This folder contains the compiled dependencies
RUN --mount=type=cache,id=amd64,target=/root/.cargo/git --mount=type=cache,id=amd64,target=/root/.cargo/registry cargo build --release --target=x86_64-unknown-linux-gnu \
    && find . -not -path "./target*" -delete

# Copies the complete project
# To avoid copying unneeded files, use .dockerignore
COPY ./ ./

# Make sure that we actually build the project
RUN touch src/main.rs

# Builds again, this time it'll just be
# your actual source files being built
RUN --mount=type=cache,id=amd64,target=/root/.cargo/git --mount=type=cache,id=amd64,target=/root/.cargo/registry cargo build --release --target=x86_64-unknown-linux-gnu


#
# Rust arm64 builder
#
FROM --platform=${BUILDPLATFORM} builder as builder-arm64

# Builds your dependencies and removes the
# dummy project, except the target folder
# This folder contains the compiled dependencies
RUN --mount=type=cache,id=arm64,target=/root/.cargo/git --mount=type=cache,id=arm64,target=/root/.cargo/registry cargo build --release --target=aarch64-unknown-linux-gnu \
    && find . -not -path "./target*" -delete

# Copies the complete project
# To avoid copying unneeded files, use .dockerignore
COPY ./ ./

# Make sure that we actually build the project
RUN touch src/main.rs

# Builds again, this time it'll just be
# your actual source files being built
RUN --mount=type=cache,id=arm64,target=/root/.cargo/git --mount=type=cache,id=arm64,target=/root/.cargo/registry cargo build --release --target=aarch64-unknown-linux-gnu


#
# Runtime for amd64
#
FROM --platform=linux/amd64 debian:bullseye-slim as runtime-amd64
WORKDIR /resback/

# Copy files needed for runtime
COPY --from=builder-amd64 /usr/src/resback/target/x86_64-unknown-linux-gnu/release/resback ./
COPY --from=builder-amd64 /usr/src/resback/.env ./.env
COPY --from=builder-amd64 /usr/src/resback/private_key.pem ./private_key.pem
COPY --from=builder-amd64 /usr/src/resback/public_key.pem ./public_key.pem


#
# Runtime for arm64
#
FROM --platform=linux/arm64 debian:bullseye-slim as runtime-arm64
WORKDIR /resback/

# Copy files needed for runtime
COPY --from=builder-arm64 /usr/src/resback/target/aarch64-unknown-linux-gnu/release/resback ./
COPY --from=builder-arm64 /usr/src/resback/.env ./.env
COPY --from=builder-arm64 /usr/src/resback/private_key.pem ./private_key.pem
COPY --from=builder-arm64 /usr/src/resback/public_key.pem ./public_key.pem


#
# Runtime for a target architecture
#
# Must be built via docker buildx.
#
FROM runtime-${TARGETARCH}
CMD [ "./resback" ]
