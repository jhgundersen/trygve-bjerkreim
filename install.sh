#!/bin/sh
set -e

REPO="jhgundersen/trygve-bjerkreim"
BINARY="tbv"
INSTALL_DIR="${HOME}/.local/bin"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64)          TARGET="x86_64-unknown-linux-musl" ;;
      aarch64 | arm64) TARGET="aarch64-unknown-linux-musl" ;;
      *) echo "Ikkje støtta arkitektur: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64)  TARGET="aarch64-apple-darwin" ;;
      *) echo "Ikkje støtta arkitektur: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *) echo "Ikkje støtta OS: $OS" >&2; exit 1 ;;
esac

# Resolve latest release tag
LATEST=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Fann ingen release på GitHub." >&2
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TARGET}"

echo "Installerer tbv ${LATEST} (${TARGET}) …"
mkdir -p "$INSTALL_DIR"
curl -sSfL "$URL" -o "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"

echo ""
echo "Ferdig! tbv er installert i $INSTALL_DIR/$BINARY"

# Warn if INSTALL_DIR is not on PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "NB: $INSTALL_DIR er ikkje i PATH. Legg til ei linje i .bashrc:" \
     && echo "    export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac
