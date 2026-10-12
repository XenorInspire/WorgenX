FROM almalinux:9

ARG NFPM_VERSION=2.47.0

ENV CARGO_HOME=/opt/cargo \
    RUSTUP_HOME=/opt/rustup \
    PATH=/opt/cargo/bin:${PATH}

RUN dnf install --allowerasing -y ca-certificates curl gcc gcc-c++ make tar gzip python3 \
    && dnf clean all

RUN curl --fail --location --silent --show-error \
        https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init \
        --output /tmp/rustup-init \
    && chmod +x /tmp/rustup-init \
    && /tmp/rustup-init -y --profile minimal --default-toolchain stable --no-modify-path \
    && rm /tmp/rustup-init

RUN curl --fail --location --silent --show-error \
        "https://github.com/goreleaser/nfpm/releases/download/v${NFPM_VERSION}/nfpm_${NFPM_VERSION}_Linux_x86_64.tar.gz" \
        | tar -xz -C /usr/local/bin nfpm

WORKDIR /workspace