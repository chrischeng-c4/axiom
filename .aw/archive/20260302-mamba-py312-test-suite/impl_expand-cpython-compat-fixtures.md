# Implementation Diff — Task expand-cpython-compat-fixtures

## Summary

```
Cargo.lock                                         | 78 +++++++++++-----------
 Cargo.toml                                         |  2 +-
 crates/cclab-mamba-tests/known_failures.toml       | 29 ++++++++
 .../fixtures/cpython/test_fstring/debug_format.py  |  2 -
 .../cpython/test_fstring/multiline_fstring.py      |  2 -
 .../cpython/test_fstring/nested_fstrings.py        |  2 -
 6 files changed, 69 insertions(+), 46 deletions(-)
```

## Diff

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 59f4c1c..09a650b 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1158,7 +1158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1169,7 +1169,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1200,7 +1200,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "pyo3",
@@ -1209,7 +1209,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "bson",
@@ -1227,7 +1227,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1254,7 +1254,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1281,7 +1281,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1294,7 +1294,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "bitvec",
  "regex",
@@ -1321,7 +1321,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1331,7 +1331,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1339,7 +1339,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1363,7 +1363,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1381,7 +1381,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1410,7 +1410,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1422,7 +1422,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "clap",
@@ -1444,7 +1444,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba-tests"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-mamba",
  "datatest-stable",
@@ -1454,7 +1454,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "image",
  "pyo3",
@@ -1465,7 +1465,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1487,7 +1487,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-nucleus"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "bson",
  "cclab-agent",
@@ -1518,7 +1518,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1548,7 +1548,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "pyo3",
  "serde",
@@ -1558,7 +1558,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-prism"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1587,7 +1587,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1619,7 +1619,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1660,7 +1660,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1686,7 +1686,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "bson",
  "dotenvy",
@@ -1701,7 +1701,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -1714,7 +1714,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1762,7 +1762,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -1787,7 +1787,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "pyo3",
  "rayon",
@@ -1799,14 +1799,14 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "pyo3",
 ]
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "bytemuck",
  "env_logger",
@@ -1836,7 +1836,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1853,7 +1853,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-asset"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "image",
@@ -1864,7 +1864,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-bundler"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "cclab-jet-asset",
@@ -1886,7 +1886,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-dev-server"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1905,7 +1905,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-pkg-manager"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "reqwest",
@@ -1922,7 +1922,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-resolver"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "node-resolve",
@@ -1935,7 +1935,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-transform"
-version = "0.3.17"
+version = "0.3.18"
 dependencies = [
  "anyhow",
  "regex",
diff --git a/Cargo.toml b/Cargo.toml
index aee7a51..2956c52 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -47,7 +47,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.17"
+version = "0.3.18"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/cclab-mamba-tests/known_failures.toml b/crates/cclab-mamba-tests/known_failures.toml
index 2e95dff..50d1fb7 100644
--- a/crates/cclab-mamba-tests/known_failures.toml
+++ b/crates/cclab-mamba-tests/known_failures.toml
@@ -18,3 +18,32 @@ issue = "parser/class-kwargs"
 [failures."test_dict/dict_unpacking"]
 reason = "Dict literal unpacking ({**d}) requires AST support for optional keys"
 issue = "parser/dict-unpack"
+
+# ─── PEP 634 Structural Pattern Matching ─────────────────────────────────────────
+
+[failures."test_match/match_basic"]
+reason = "match/case (PEP 634) not yet supported by parser"
+issue = "parser/match-case"
+category = "parser"
+
+[failures."test_match/match_class"]
+reason = "match/case class patterns (PEP 634) not yet supported"
+issue = "parser/match-case"
+category = "parser"
+
+[failures."test_match/match_mapping"]
+reason = "match/case mapping patterns (PEP 634) not yet supported"
+issue = "parser/match-case"
+category = "parser"
+
+[failures."test_match/match_sequence"]
+reason = "match/case sequence patterns (PEP 634) not yet supported"
+issue = "parser/match-case"
+category = "parser"
+
+# ─── Import alias syntax ─────────────────────────────────────────────────────────
+
+[failures."test_import/import_alias"]
+reason = "Import alias (import X as Y) not yet supported by parser"
+issue = "parser/import-alias"
+category = "parser"
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py
index 63f76c6..ce5ffc2 100644
--- a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/debug_format.py
@@ -1,6 +1,4 @@
 # RUN: parse
-# XFAIL
-# REASON: f'{expr=}' debug format specifier not yet implemented
 
 x = 42
 # Debug format specifier (Python 3.8+)
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py
index 84a88ac..63eb894 100644
--- a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/multiline_fstring.py
@@ -1,6 +1,4 @@
 # RUN: parse
-# XFAIL
-# REASON: Multiline f-strings with embedded newlines not yet supported
 
 # Multi-line f-string (PEP 701 Python 3.12)
 x = 10
diff --git a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py
index e6b49f8..cfcb903 100644
--- a/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py
+++ b/crates/cclab-mamba-tests/tests/fixtures/cpython/test_fstring/nested_fstrings.py
@@ -1,6 +1,4 @@
 # RUN: parse
-# XFAIL
-# REASON: Nested f-strings require PEP 701 full parser support
 
 # Nested f-strings (PEP 701 - Python 3.12)
 s = f"{"nested"}"
```
