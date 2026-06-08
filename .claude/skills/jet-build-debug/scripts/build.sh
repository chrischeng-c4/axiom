#!/bin/bash
set -e

# Use git root as project directory
cd "$(git rev-parse --show-toplevel)"

cargo build -p jet && \
  rm -f ~/.cargo/bin/jet && cp target/debug/jet ~/.cargo/bin/jet && chmod +x ~/.cargo/bin/jet && \
  codesign -s - -f ~/.cargo/bin/jet 2>/dev/null || true

echo "Installed: $(~/.cargo/bin/jet --version 2>/dev/null || echo 'jet')"

echo ""
echo "Build complete."
