#!/usr/bin/env bash
set -e

cd /workspace

readonly TARGET=x86_64-unknown-linux-gnu
readonly RELEASE_DIR="$CARGO_TARGET_DIR/$TARGET/release"

VERSION="$(cargo metadata --locked --no-deps --format-version 1 \
    | python3 -c 'import json, sys; print(next(package["version"] for package in json.load(sys.stdin)["packages"] if package["name"] == "worgen_x"))')"
export VERSION

# Compress package documentation without modifying the read-only source tree.
readonly PACKAGE_RESOURCES_DIR=/tmp/worgenx-package-resources
mkdir -p \
    "$PACKAGE_RESOURCES_DIR/usr/share/doc/worgenx" \
    "$PACKAGE_RESOURCES_DIR/usr/share/man/man1"

gzip -9 -c /workspace/resources/usr/share/doc/worgenx/changelog \
    > "$PACKAGE_RESOURCES_DIR/usr/share/doc/worgenx/changelog.gz"
gzip -9 -c /workspace/resources/usr/share/man/man1/worgenx.1 \
    > "$PACKAGE_RESOURCES_DIR/usr/share/man/man1/worgenx.1.gz"
gzip -9 -c /workspace/resources/usr/share/man/man1/worgenx_gui.1 \
    > "$PACKAGE_RESOURCES_DIR/usr/share/man/man1/worgenx_gui.1.gz"

# Build each mutually exclusive feature separately and strip the packaged copy.
for mode in cli gui; do
    cargo build \
        --locked \
        --release \
        --features "$mode" \
        --target "$TARGET"
    if [[ "$mode" == "cli" ]]; then
        install -s -D -m 0755 "$RELEASE_DIR/worgen_x" "/tmp/worgenx"
    else
        install -s -D -m 0755 "$RELEASE_DIR/worgen_x" "/tmp/worgenx_gui"
    fi

done

nfpm package \
    --config /workspace/packaging/nfpm.yaml \
    --packager deb \
    --target "/output/worgenx_${VERSION}_amd64.deb"

nfpm package \
    --config /workspace/packaging/nfpm.yaml \
    --packager rpm \
    --target "/output/worgenx_${VERSION}_x86_64.rpm"