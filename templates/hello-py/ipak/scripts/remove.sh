#!/bin/bash

set -e

# Ensure project name, version, and remove mode are set
if [ -z "$IPAK_PROJECT_NAME" ] || [ -z "$IPAK_PROJECT_VERSION" ]; then
    echo "Error: IPAK_PROJECT_NAME and IPAK_PROJECT_VERSION must be set" >&2
    exit 1
fi
if [ -z "$IPAK_REMOVE_MODE" ]; then
    echo "Error: IPAK_REMOVE_MODE must be set (local or global)" >&2
    exit 1
fi

# Validate remove mode
case "$IPAK_REMOVE_MODE" in
    local|global)
        ;;
    *)
        echo "Error: Invalid IPAK_REMOVE_MODE: $IPAK_REMOVE_MODE (must be 'local' or 'global')" >&2
        exit 1
        ;;
esac

# Determine binary path
if [ "$IPAK_REMOVE_MODE" = "local" ]; then
    BINARY_PATH="$HOME/.ipak/bin/$IPAK_PROJECT_NAME"
else
    BINARY_PATH="/usr/local/bin/$IPAK_PROJECT_NAME"
fi

echo "Removing $IPAK_PROJECT_NAME version $IPAK_PROJECT_VERSION from $BINARY_PATH"

# Remove binary if it exists
if [ -f "$BINARY_PATH" ]; then
    rm "$BINARY_PATH"
    echo "Removed successfully"
else
    echo "Warning: Binary '$BINARY_PATH' not found, nothing to remove"
fi