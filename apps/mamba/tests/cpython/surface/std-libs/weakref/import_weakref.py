# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "import_weakref"
# subject = "weakref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref: import_weakref (surface)."""
import weakref

assert hasattr(weakref, "ref")
print("import_weakref OK")
