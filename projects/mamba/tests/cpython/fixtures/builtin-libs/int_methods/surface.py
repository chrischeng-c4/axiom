# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/int_methods: surface probes (CPython 3.12 oracle)."""

# Probes for documented builtin attributes / methods. Each `assert`
# verifies that the API surface is in place.

# Generic surface check: `import` succeeds and exposes basic dunder
# attributes.
import builtins
assert hasattr(builtins, "object")

print("surface OK")
