# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "import_cmath"
# subject = "cmath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: import_cmath (surface)."""
import cmath

assert hasattr(cmath, "sqrt")
print("import_cmath OK")
