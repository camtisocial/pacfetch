#!/bin/bash
# an arch environment with pending updates for testing pacfetch -Syu/-Sy/-Su 
#
# Usage:
#   ./test.sh -Syu
#   ./test.sh bash         # Drop into shell for manual testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building pacfetch..."
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

IMAGE_NAME="pacfetch-test"
if [[ -z "$(docker images -q $IMAGE_NAME 2>/dev/null)" ]]; then
    echo "Building test image..."
    docker build -t "$IMAGE_NAME" "$SCRIPT_DIR"
fi

PACMAN_CONF="/etc/pacman.conf"

if [[ "$1" == "bash" ]]; then
    docker run -it --rm --privileged \
        -v "$PROJECT_DIR/target/release/pacfetch:/usr/local/bin/pacfetch:ro" \
        -v "$PACMAN_CONF:/etc/pacman.conf:ro" \
        --entrypoint bash \
        "$IMAGE_NAME"
else
    docker run -it --rm --privileged \
        -v "$PROJECT_DIR/target/release/pacfetch:/usr/local/bin/pacfetch:ro" \
        -v "$PACMAN_CONF:/etc/pacman.conf:ro" \
        "$IMAGE_NAME" \
        "$@"
fi
