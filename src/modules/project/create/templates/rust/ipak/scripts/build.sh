#!/bin/bash

set -e

# Ensure project name and version are set
if [ -z "$IPAK_PROJECT_NAME" ] || [ -z "$IPAK_PROJECT_VERSION" ]; then
    echo "IpakError: IPAK_PROJECT_NAME and IPAK_PROJECT_VERSION must be set" >&2
    exit 1
fi

# Default to debug mode if IPAK_BUILD_MODE is unset
BUILD_MODE="${IPAK_BUILD_MODE:-debug}"

# Validate build mode
case "$BUILD_MODE" in
    release|debug)
        ;;
    *)
        echo "IpakError: Invalid IPAK_BUILD_MODE: $BUILD_MODE (must be 'release' or 'debug')" >&2
        exit 1
        ;;
esac

echo "Building $IPAK_PROJECT_NAME version $IPAK_PROJECT_VERSION in $BUILD_MODE mode"

# Run cargo build
if [ "$BUILD_MODE" = "release" ]; then
    cargo build --release
else
    cargo build
fi

echo "Build completed successfully"