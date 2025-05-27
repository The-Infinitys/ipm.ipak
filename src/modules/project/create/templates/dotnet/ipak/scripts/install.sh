#!/bin/bash

set -e

# Ensure project name, version, and install mode are set
if [ -z "$IPAK_PROJECT_NAME" ] || [ -z "$IPAK_PROJECT_VERSION" ]; then
    echo "Error: IPAK_PROJECT_NAME and IPAK_PROJECT_VERSION must be set" >&2
    exit 1
fi
if [ -z "$IPAK_INSTALL_MODE" ]; then
    echo "Error: IPAK_INSTALL_MODE must be set (local or global)" >&2
    exit 1
fi

# Validate modes
case "$IPAK_INSTALL_MODE" in
    local|global)
        ;;
    *)
        echo "Error: Invalid IPAK_INSTALL_MODE: $IPAK_INSTALL_MODE (must be 'local' or 'global')" >&2
        exit 1
        ;;
esac
# Determine source and destination paths
BINARY_DIR="target/release"
BINARY_PATH="$BINARY_DIR/$IPAK_PROJECT_NAME"
if [ "$IPAK_INSTALL_MODE" = "local" ]; then
    INSTALL_DIR="$HOME/.ipak/bin"
else
    INSTALL_DIR="/usr/local/bin"
fi
DEST_PATH="$INSTALL_DIR/$IPAK_PROJECT_NAME"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary '$BINARY_PATH' not found. Run build first." >&2
    exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"

echo "Installing $IPAK_PROJECT_NAME version $IPAK_PROJECT_VERSION to $DEST_PATH"

# Copy binary
cp "$BINARY_PATH" "$DEST_PATH"

# Set executable permissions
chmod +x "$DEST_PATH"

echo "Installation completed successfully"