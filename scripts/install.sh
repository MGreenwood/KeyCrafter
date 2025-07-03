#!/bin/bash

# Determine OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    BINARY="keycrafter-darwin-x64"
else
    BINARY="keycrafter-linux-x64"
fi

# Create install directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Check for existing installation
if [ -f "$INSTALL_DIR/keycrafter" ]; then
    echo "Found existing KeyCrafter installation. Removing..."
    killall keycrafter 2>/dev/null || true
    rm -f "$INSTALL_DIR/keycrafter"
fi

echo "Downloading KeyCrafter..."
curl -L "https://play.keycrafter.fun/$BINARY" -o "$INSTALL_DIR/keycrafter"
chmod +x "$INSTALL_DIR/keycrafter"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_RC="$HOME/.$(basename $SHELL)rc"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_RC"
    echo "Added KeyCrafter to your PATH in $SHELL_RC"
fi

echo "
KeyCrafter installed successfully!
You can now run 'keycrafter' from any terminal.
Note: You may need to restart your terminal or run 'source $SHELL_RC' for the PATH changes to take effect." 