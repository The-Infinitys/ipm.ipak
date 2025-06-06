#!/bin/bash

set -e # Exit immediately if a command exits with a non-zero status.

# Ensure IPAK_PROJECT_NAME is set
if [ -z "$IPAK_PROJECT_NAME" ]; then
    echo "Error: IPAK_PROJECT_NAME must be set." >&2
    exit 1
fi

RELEASE_BINARY="./target/release/$IPAK_PROJECT_NAME"
DEBUG_BINARY="./target/debug/$IPAK_PROJECT_NAME"

release_exists=false
debug_exists=false

if [ -f "$RELEASE_BINARY" ]; then
    release_exists=true
fi

if [ -f "$DEBUG_BINARY" ]; then
    debug_exists=true
fi

if $release_exists && $debug_exists; then
    echo "Both release and debug binaries found. Checking which is newer."
    if [ "$RELEASE_BINARY" -nt "$DEBUG_BINARY" ]; then
        echo "Executing release binary (newer): $RELEASE_BINARY $*"
        "$RELEASE_BINARY" "$@"
    else
        echo "Executing debug binary (newer or same age): $DEBUG_BINARY $*"
        "$DEBUG_BINARY" "$@"
    fi
elif $release_exists; then
    echo "Executing release binary: $RELEASE_BINARY $*"
    "$RELEASE_BINARY" "$@"
elif $debug_exists; then
    echo "Executing debug binary: $DEBUG_BINARY $*"
    "$DEBUG_BINARY" "$@"
else
    echo "Error: Neither release ('$RELEASE_BINARY') nor debug ('$DEBUG_BINARY') binary found." >&2
    echo "Please build the project first." >&2
    exit 1
fi
