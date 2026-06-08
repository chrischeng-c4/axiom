# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/equality_hash: language-area surface probes (CPython 3.12 oracle)."""

# Core language constructs always available.
import builtins
assert hasattr(builtins, "object")
assert hasattr(builtins, "type")
assert hasattr(builtins, "list")
assert hasattr(builtins, "dict")
assert hasattr(builtins, "set")
assert hasattr(builtins, "tuple")
assert hasattr(builtins, "str")
assert hasattr(builtins, "bytes")
assert hasattr(builtins, "int")
assert hasattr(builtins, "float")
assert hasattr(builtins, "complex")
assert hasattr(builtins, "bool")
assert hasattr(builtins, "isinstance")
assert hasattr(builtins, "issubclass")
assert hasattr(builtins, "callable")

print("surface OK")
