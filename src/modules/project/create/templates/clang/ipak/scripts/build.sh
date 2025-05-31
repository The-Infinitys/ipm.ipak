#!/bin/bash

set -e

# Ensure project name and version are set
if [ -z "$IPAK_PROJECT_NAME" ] || [ -z "$IPAK_PROJECT_VERSION" ]; then
    echo "Error: IPAK_PROJECT_NAME and IPAK_PROJECT_VERSION must be set" >&2
    exit 1
fi

# Default to debug mode if IPAK_BUILD_MODE is unset
BUILD_MODE="${IPAK_BUILD_MODE:-debug}"

# Validate build mode
case "$BUILD_MODE" in
release | debug) ;;
*)
    echo "Error: Invalid IPAK_BUILD_MODE: $BUILD_MODE (must be 'release' or 'debug')" >&2
    exit 1
    ;;
esac

echo "Building $IPAK_PROJECT_NAME version $IPAK_PROJECT_VERSION in $BUILD_MODE mode"
# Create build directory if it doesn't exist and navigate into it
# Remove build directory if it exists
if [ -d "build" ]; then
    echo "Removing existing build directory"
    rm -rf build
fi

# Create build directory and navigate into it

mkdir -p build
cd build

# Configure CMake based on build mode

if [ "$BUILD_MODE" = "release" ]; then
    cmake .. -DCMAKE_BUILD_TYPE=Debug
else
    cmake .. -DCMAKE_BUILD_TYPE=Release
fi

cmake --build .
cd ..

echo "Build completed successfully"
# build binary
