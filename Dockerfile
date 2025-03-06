# syntax=docker/dockerfile:experimental
# Copyright 2025 SCTG Development - Ronan LE MEILLAT
# SPDX-License-Identifier: AGPL-3.0-or-later
FROM ubuntu:noble AS builder
RUN apt-get update && apt-get install --no-install-recommends -y curl build-essential debhelper devscripts \
                pkg-config libssl-dev libc-dev libstdc++-13-dev libgcc-13-dev \
                zip git libcurl4-openssl-dev musl-dev musl-tools cmake libclang-dev g++
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 
RUN echo $(dpkg --print-architecture)
RUN mkdir /build
RUN if [ "$(dpkg --print-architecture)" = "armhf" ]; then \
       . /root/.cargo/env && rustup target add armv7-unknown-linux-musleabihf; \
       ln -svf /usr/bin/ar /usr/bin/arm-linux-musleabihf-ar; \
       ln -svf /usr/bin/strip /usr/bin/arm-linux-musleabihf-strip; \
       ln -svf /usr/bin/ranlib /usr/bin/arm-linux-musleabihf-ranlib; \
       echo "armv7-unknown-linux-musleabihf" > /build/_target ; \
    fi
RUN if [ "$(dpkg --print-architecture)" = "arm64" ]; then \
       . /root/.cargo/env && rustup target add aarch64-unknown-linux-musl; \
       ln -svf /usr/bin/ar /usr/bin/aarch64-linux-musl-ar; \
       ln -svf /usr/bin/strip /usr/bin/aarch64-linux-musl-strip; \
       ln -svf /usr/bin/ranlib /usr/bin/aarch64-linux-musl-ranlib; \
       echo "aarch64-unknown-linux-musl" > /build/_target ; \
    fi
RUN if [ "$(dpkg --print-architecture)" = "amd64" ]; then \
       . /root/.cargo/env && rustup target add x86_64-unknown-linux-musl; \
       echo "x86_64-unknown-linux-musl" > /build/_target ; \
    fi

COPY src /build/src
COPY Cargo.toml /build/Cargo.toml


RUN export TARGET=$(cat /build/_target) \
    && . /root/.cargo/env && cd /build \ 
    && cargo build --target=$TARGET --release || (echo "Build failed, entering sleep mode for debugging..." && cp -av /root/.cargo /build/ && exit 1) \
    && mkdir -p /build/ubuntu-noble/bin \
    && cp /build/target/$(cat /build/_target)/release/vsixHarvester /build/ubuntu-noble/bin/
FROM ubuntu:noble
COPY --from=builder /build/ubuntu-noble/bin/vsixHarvester /usr/local/bin/vsixHarvester
ENTRYPOINT [ "/usr/local/bin/vsixHarvester" ]