#!/usr/bin/env bash
set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/dist"
IMAGE_NAME="${WORGENX_PACKAGER_IMAGE:-worgenx-linux-packager:local}"

if [[ "$(uname -m)" != "x86_64" ]]; then
    printf 'Error: this script must run on an x86_64 host.\n' >&2
    exit 1
fi

if command -v docker >/dev/null 2>&1; then
    CONTAINER_RUNTIME=docker
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_RUNTIME=podman
else
    printf 'Error: install Docker or Podman before running this script.\n' >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

"$CONTAINER_RUNTIME" build \
    --platform linux/amd64 \
    --file "$ROOT_DIR/build/linux-packager.Dockerfile" \
    --tag "$IMAGE_NAME" \
    "$ROOT_DIR"

RUN_ARGS=(
    run --rm
    --platform linux/amd64
    --env HOME=/tmp
    --env CARGO_HOME=/tmp/cargo-home
    --env RUSTUP_HOME=/opt/rustup
    --env CARGO_TARGET_DIR=/tmp/worgenx-target
    --volume "$ROOT_DIR:/workspace:ro"
    --volume "$OUTPUT_DIR:/output"
    --workdir /workspace
    "$IMAGE_NAME"
)

"$CONTAINER_RUNTIME" "${RUN_ARGS[@]}" /bin/bash /workspace/build/package-linux-in-container.sh

printf '\nLinux packages created in %s\n' "$OUTPUT_DIR"