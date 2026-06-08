#!/bin/bash
set -e

# Use git root as project directory
cd "$(git rev-parse --show-toplevel)"

cargo build -p mamba --bin mamba && \
  rm -f ~/.cargo/bin/mamba && cp target/debug/mamba ~/.cargo/bin/mamba && chmod +x ~/.cargo/bin/mamba && \
  codesign -s - -f ~/.cargo/bin/mamba 2>/dev/null || true

echo "Installed: $(~/.cargo/bin/mamba --version 2>/dev/null || echo 'mamba')"

echo ""
echo "Build complete (debug)."
