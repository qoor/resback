FROM rust:slim-bullseye as builder
WORKDIR /usr/src/resback
COPY ./ ./
RUN apt update && apt install -y libssl-dev pkg-config
ENV SQLX_OFFLINE true
RUN cargo install --path ./

FROM debian:bullseye-slim
RUN mkdir -p /resback
WORKDIR /resback/
COPY --from=builder /usr/local/cargo/bin/resback ./
COPY --from=builder /usr/src/resback/.env ./.env
COPY --from=builder /usr/src/resback/private_key.pem ./private_key.pem
COPY --from=builder /usr/src/resback/public_key.pem ./public_key.pem
CMD [ "./resback" ]
