# FROM messense/rust-musl-cross:x86_64-musl AS chef
FROM clux/muslrust:1.71.1-stable AS chef
ENV TZ=Asia/Shanghai
RUN cargo install cargo-chef
WORKDIR /build

RUN ln -fs /usr/share/zoneinfo/$TZ /etc/localtime
RUN sed -i 's|archive.ubuntu.com|mirrors.ustc.edu.cn|g' /etc/apt/sources.list
RUN apt-get update && apt-get install -y tzdata

FROM chef AS planner
COPY . .
# Generate info for caching dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
ARG APP_VERSION=v1.0.0
# Build application
RUN APP_VERSION=${APP_VERSION} BUILD_TIME=`date +%Y-%m-%dT%H:%M:%S` cargo build --release --target x86_64-unknown-linux-musl

# Create a new stage with a minimal image
FROM alpine:3.17.5
ENV TZ=Asia/Shanghai

WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/service-demo .
ENTRYPOINT ["./service-demo"]
