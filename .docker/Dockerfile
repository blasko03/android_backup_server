FROM --platform=$BUILDPLATFORM rust:1.92 AS build
ARG TARGETARCH
ARG TARGETOS
ARG BUILD_TYPE

ENV PKG_CONFIG_ALLOW_CROSS=true
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

WORKDIR /source
COPY . .

RUN mkdir /build && mkdir /build/lib && mkdir /build/app

RUN case "$TARGETARCH" in \
          arm64)   ARCH="aarch64" ;; \
          aarch64) ARCH="arm64" ;; \
          x86_64)  ARCH="x86_64" ;; \
          amd64)   ARCH="x86_64" ;; \
        esac && \
    dpkg --add-architecture $TARGETARCH && \
    apt update &&  \
    apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross musl:$TARGETARCH musl-tools:$TARGETARCH musl-dev:$TARGETARCH && \
    rustup target add $ARCH-unknown-$TARGETOS-$BUILD_TYPE && \
    rustup toolchain install stable-$ARCH-unknown-linux-$BUILD_TYPE --force-non-host && \
    cargo build --release --target $ARCH-unknown-$TARGETOS-$BUILD_TYPE && \
    cp /source/target/$ARCH-unknown-$TARGETOS-$BUILD_TYPE/release/android_backup_server /build/app/android_backup_server

RUN if [ "$BUILD_TYPE" = "gnu" ]; then \
        if [ "$TARGETARCH" = "arm64" ]; then \
            mkdir /build/lib/aarch64-linux-gnu && \
            cp /usr/aarch64-linux-gnu/lib/libgcc_s.so.1 /build/lib/aarch64-linux-gnu/libgcc_s.so.1 && \
            cp /usr/aarch64-linux-gnu/lib/libm.so.6 /build/lib/aarch64-linux-gnu/libm.so.6 && \
            cp /usr/aarch64-linux-gnu/lib/libc.so.6 /build/lib/aarch64-linux-gnu/libc.so.6 && \
            cp /usr/aarch64-linux-gnu/lib/ld-linux-aarch64.so.1 /build/lib/ld-linux-aarch64.so.1 ; \
        else \
            mkdir /build/lib/x86_64-linux-gnu && mkdir /build/lib64 && \
            cp /lib/x86_64-linux-gnu/libgcc_s.so.1 /build/lib/x86_64-linux-gnu/libgcc_s.so.1 && \
            cp /lib/x86_64-linux-gnu/libm.so.6 /build/lib/x86_64-linux-gnu/libm.so.6 && \
            cp /lib/x86_64-linux-gnu/libc.so.6 /build/lib/x86_64-linux-gnu/libc.so.6 && \
            cp /lib64/ld-linux-x86-64.so.2 /build/lib64/ld-linux-x86-64.so.2 ; \
        fi ; \
    fi


FROM scratch AS app
ENV RUST_LOG=info
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
RUN mkdir -p /tmp && chmod 777 /tmp
WORKDIR /app

COPY --from=build $SSL_CERT_FILE $SSL_CERT_FILE
COPY --from=build /build/ /

ENTRYPOINT ["/app/android_backup_server"]