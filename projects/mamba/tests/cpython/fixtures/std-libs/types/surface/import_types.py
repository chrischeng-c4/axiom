# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "import_types"
# subject = "types"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types: import_types (surface)."""
import types

assert hasattr(types, "SimpleNamespace")
print("import_types OK")
