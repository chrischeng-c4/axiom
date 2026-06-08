# Implementation Diff

## Summary

```
install.sh | 46 ++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)
```

## Diff

```diff
diff --git a/install.sh b/install.sh
new file mode 100755
index 0000000..47ac614
--- /dev/null
+++ b/install.sh
@@ -0,0 +1,46 @@
+#!/bin/bash
+set -e
+
+cd "$(git rev-parse --show-toplevel)"
+
+echo "=== cclab install ==="
+echo ""
+
+# 1. Check/install prerequisites
+if ! command -v rustup >/dev/null 2>&1; then
+    echo "Installing rustup..."
+    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
+    source "$HOME/.cargo/env"
+fi
+echo "rustup: $(rustup --version 2>&1 | head -1)"
+echo "cargo:  $(cargo --version)"
+
+if ! command -v uv >/dev/null 2>&1; then
+    echo "Installing uv..."
+    curl -LsSf https://astral.sh/uv/install.sh | sh
+fi
+echo "uv:     $(uv --version)"
+echo ""
+
+# 2. Build cclab CLI
+echo "Building cclab-cli..."
+cargo build -p cclab-cli
+
+# 3. Install binary
+rm -f ~/.cargo/bin/cclab
+cp target/debug/cclab ~/.cargo/bin/cclab
+chmod +x ~/.cargo/bin/cclab
+echo "Installed: $(~/.cargo/bin/cclab --version)"
+echo ""
+
+# 4. Restart MCP server
+echo "Restarting MCP server..."
+cclab server shutdown 2>/dev/null || true
+lsof -ti :3456 | xargs kill 2>/dev/null || true
+sleep 1
+cclab server start
+
+# 5. Done
+echo ""
+echo "cclab installed successfully."
+echo "Remember to restart Claude Code to pick up new MCP tools."
```
