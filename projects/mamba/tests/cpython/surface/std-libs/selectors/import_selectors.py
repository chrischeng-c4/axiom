# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "import_selectors"
# subject = "selectors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors: import_selectors (surface)."""
import selectors

assert hasattr(selectors, "DefaultSelector")
print("import_selectors OK")
