#!/bin/bash

set -e

# Ensure project name, version, and purge mode are set
if [ -z "$IPAK_PROJECT_NAME" ] || [ -z "$IPAK_PROJECT_VERSION" ]; then
    echo "Error: IPAK_PROJECT_NAME and IPAK_PROJECT_VERSION must be set" >&2
    exit 1
fi
if [ -z "$IPAK_PURGE_MODE" ]; then
    echo "Error: IPAK_PURGE_MODE must be set (local or global)" >&2
    exit 1
fi

# Validate purge mode
case "$IPAK_PURGE_MODE" in
    local|global)
        ;;
    *)
        echo "Error: Invalid IPAK_PURGE_MODE: $IPAK_PURGE_MODE (must be 'local' or 'global')" >&2
        exit 1
        ;;
esac

# Determine binary path
if [ "$IPAK_PURGE_MODE" = "local" ]; then
    BINARY_PATH="$HOME/.ipak/bin/$IPAK_PROJECT_NAME"
else
    BINARY_PATH="/usr/local/bin/$IPAK_PROJECT_NAME"
fi

echo "Purging $IPAK_PROJECT_NAME version $IPAK_PROJECT_VERSION"

# Remove binary if it exists
if [ -f "$BINARY_PATH" ]; then
    rm "$BINARY_PATH"
    echo "Removed binary: $BINARY_PATH"
else
    echo "Warning: Binary '$BINARY_PATH' not found"
fi

echo "Purge completed successfully"