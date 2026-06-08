# Implementation Diff

## Summary

```
.../skills/cclab-release-patch/scripts/release.sh  |  3 +
 Cargo.lock                                         | 70 +++++++++++-----------
 Cargo.toml                                         |  2 +-
 3 files changed, 39 insertions(+), 36 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/cclab-release-patch/scripts/release.sh b/.claude/skills/cclab-release-patch/scripts/release.sh
index 0affd13..728a84e 100755
--- a/.claude/skills/cclab-release-patch/scripts/release.sh
+++ b/.claude/skills/cclab-release-patch/scripts/release.sh
@@ -19,6 +19,9 @@ echo "Bumping version: $CURRENT_VERSION → $NEW_VERSION"
 # Update Cargo.toml
 sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
 
+# Sync Cargo.lock so cargo detects the version change and recompiles
+cargo update -w 2>/dev/null || cargo generate-lockfile
+
 # Build and install
 cargo build -p cclab-cli && rm -f ~/.cargo/bin/cclab && cp target/debug/cclab ~/.cargo/bin/cclab && chmod +x ~/.cargo/bin/cclab
 
diff --git a/Cargo.lock b/Cargo.lock
index 1f1658c..b360ac7 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1202,7 +1202,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli-registry"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "clap",
@@ -1211,7 +1211,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1220,7 +1220,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "bson",
@@ -1238,7 +1238,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1265,7 +1265,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1292,7 +1292,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1305,7 +1305,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "bitvec",
  "regex",
@@ -1332,7 +1332,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1342,7 +1342,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1350,7 +1350,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1374,7 +1374,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1392,7 +1392,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1431,7 +1431,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1460,7 +1460,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1472,7 +1472,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "base64 0.22.1",
@@ -1500,7 +1500,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "image",
  "pyo3",
@@ -1511,7 +1511,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1533,7 +1533,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1564,7 +1564,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1594,7 +1594,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "pyo3",
  "serde",
@@ -1604,7 +1604,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1633,7 +1633,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1665,7 +1665,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1706,7 +1706,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1732,7 +1732,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1747,7 +1747,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1760,7 +1760,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1808,7 +1808,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-cli"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -1834,7 +1834,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-mcp"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1875,7 +1875,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1901,7 +1901,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1913,14 +1913,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.30"
+version = "0.3.31"
 dependencies = [
  "bytemuck",
  "env_logger",
diff --git a/Cargo.toml b/Cargo.toml
index 971c17e..3eba47d 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -43,7 +43,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.30"
+version = "0.3.31"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
```
